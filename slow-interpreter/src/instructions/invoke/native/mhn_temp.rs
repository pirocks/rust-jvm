#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]


use std::sync::Arc;
use rust_jvm_common::classnames::ClassName;
use std::cell::RefCell;
use crate::interpreter_util::{check_inited_class, run_constructor, push_new_object};
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use crate::rust_jni::{get_all_methods, get_all_fields};
use crate::utils::string_obj_to_string;
use classfile_view::view::HasAccessFlags;
use rust_jvm_common::classfile::{REF_invokeVirtual, REF_invokeStatic, REF_invokeInterface, ACC_STATIC};
use crate::{JVMState, StackEntry};
use crate::instructions::invoke::static_::invoke_static_impl;
use crate::instructions::invoke::virtual_::invoke_virtual_method_i;
use std::ops::Deref;
use crate::java_values::{JavaValue, NormalObject, ArrayObject};
use crate::java_values::Object::{Object, Array};
use crate::runtime_class::RuntimeClass;
use descriptor_parser::{MethodDescriptor, parse_method_descriptor};
use crate::class_objects::get_or_create_class_object;

pub fn MHN_resolve(state: & JVMState, frame: &StackEntry, args: &mut Vec<JavaValue>) -> Option<JavaValue> {
//todo
//so as far as I can find this is undocumented.
//so as far as I can figure out we have a method name and a class
//we lookup for a matching method, throw various kinds of exceptions if it doesn't work
// and return a brand new object
//                        dbg!(&args[0]);
//     dbg!(&args[0].unwrap_object().unwrap().lookup_field("clazz"));
//     dbg!(&args[1]);
    let member_name = args[0].unwrap_object().unwrap();
    // dbg!(member_name.lookup_field("clazz"));
    // dbg!(member_name.lookup_field("name"));
    // dbg!(member_name.lookup_field("type"));
    let type_java_value = member_name.lookup_field("type");
    // dbg!(&type_java_value.unwrap_normal_object().class_pointer.class_view.name()); // so this is a string before resolution?
    // dbg!(member_name.lookup_field("flags"));
//                        let class = args[1].unwrap_object().unwrap();
//                        let name = string_obj_to_string(member_name.lookup_field("name").unwrap_object());
//todo maybe create a class for this resolution object
//todo actually do whatever I'm meant to do here.
//what openjdk has to say: methodHandles.cpp
// java_lang_invoke_MemberName::set_flags(   mname_oop, flags);
// java_lang_invoke_MemberName::set_vmtarget(mname_oop, m());
// java_lang_invoke_MemberName::set_vmindex( mname_oop, vmindex);   // vtable/itable index
// java_lang_invoke_MemberName::set_clazz(   mname_oop, m_klass->java_mirror());
// // Note:  name and type can be lazily computed by resolve_MemberName,
// // if Java code needs them as resolved String and MethodType objects.
// // The clazz must be eagerly stored, because it provides a GC
// // root to help keep alive the Method*.
// // If relevant, the vtable or itable value is stored as vmindex.
// // This is done eagerly, since it is readily available without
// // constructing any new objects.


    let resolution_object = JavaValue::Object(Arc::new(Object(NormalObject {
        gc_reachable: false,
        fields: RefCell::new(Default::default()),
        class_pointer: check_inited_class(state, &ClassName::object(), frame.class_pointer.loader.clone()),
        bootstrap_loader: true,
        class_object_ptype: RefCell::new(None),
    })).into());
    member_name.unwrap_normal_object().fields.borrow_mut().insert("resolution".to_string(), resolution_object);
    //todo sets resolution to something on failure
// private Class<?> clazz;
// private String name;
// private Object type;
// private int flags;
// private Object resolution;
    //java.lang.invoke.MemberName.Factory#resolve clones before calling us so this suggests we can edit as we desire.
    let flags_val = member_name.unwrap_normal_object().fields.borrow().get("flags").unwrap().unwrap_int();
    let is_field = flags_val & 262144 > 0;//todo these magic numbers come from MemberName(the java class where they are also magic numbers.)
    let is_method = flags_val & 65536 > 0;
    let is_constructor = flags_val & 131072 > 0;
    let clazz_field = member_name.lookup_field("clazz");
    let clazz = clazz_field.unwrap_normal_object();
    let clazz_points_to = clazz.class_object_ptype.borrow().as_ref().unwrap().unwrap_class_type();//todo doesn't work for arrays
    let clazz_as_runtime_class = check_inited_class(state, &clazz_points_to, frame.class_pointer.loader.clone());
    let name = string_obj_to_string(member_name.lookup_field("name").unwrap_object());
    let debug = &name == "checkSpreadArgument";
    let type_ = type_java_value.unwrap_normal_object();
    if is_field {
        assert!(!is_method);
        let all_fields = get_all_fields(state,frame.clone(),clazz_as_runtime_class);
        dbg!(type_);
        if type_.class_pointer.class_view.name() == ClassName::class(){
            let target_ptype = type_.class_object_ptype.borrow().as_ref().unwrap().clone();
            let (res_c, res_i) = all_fields.iter().find(|(c,i)|{
                let field = c.class_view.field(*i);
                field.field_name() == name &&
                    field.field_type() == target_ptype
            }).unwrap();

            let correct_flags = res_c.class_view.field(*res_i).access_flags();
            let new_flags = ((flags_val as u32) | (correct_flags as u32)) as i32;

            //todo do we need to update clazz?
            member_name.unwrap_normal_object().fields.borrow_mut().insert("flags".to_string(), JavaValue::Int(new_flags));
        }else {
            unimplemented!()
        }

    } else if is_method || is_constructor {
        assert!(!is_constructor);//todo not implemented yet
        assert!(!is_field);
        // frame.print_stack_trace();
        let all_methods = get_all_methods(state, frame.clone(), clazz_as_runtime_class);
        if type_.class_pointer.class_view.name() == ClassName::method_type() {
            let r_type_class = type_java_value.unwrap_object_nonnull().lookup_field("rtype").unwrap_object_nonnull();
            let param_types_class = type_java_value.unwrap_object_nonnull().lookup_field("ptypes").unwrap_array().unwrap_object_array_nonnull();
            let r_type_as_ptype = r_type_class.unwrap_normal_object().class_object_ptype.borrow().as_ref().unwrap().clone();
            let params_as_ptype: Vec<PTypeView> = param_types_class.iter().map(|x| { x.unwrap_normal_object().class_object_ptype.borrow().as_ref().unwrap().clone() }).collect();
            //todo how do the params work with static v. not static
            match all_methods.iter().find(|(x, i)| {
                let c_method = x.class_view.method_view_i(*i);
                //todo need to handle signature polymorphism here and in many places
                c_method.name() == name && if c_method.is_signature_polymorphic() {
                    c_method.desc().parameter_types.len() == 1 &&
                        c_method.desc().parameter_types[0] == PTypeView::array(PTypeView::object()).to_ptype() &&
                        c_method.desc().return_type == PTypeView::object().to_ptype()
                } else {
                    c_method.desc().parameter_types == params_as_ptype.iter().map(|x|x.to_ptype()).collect::<Vec<_>>() //todo what about overloading
                }
            }){
                None => {
                    member_name.unwrap_normal_object().fields.borrow_mut().insert("resolution".to_string(), JavaValue::Object(None));
                },
                Some((resolved_method_runtime_class, resolved_i)) => {
                    let correct_flags = resolved_method_runtime_class.class_view.method_view_i(*resolved_i).access_flags();
                    let new_flags = ((flags_val as u32) | (correct_flags as u32)) as i32;

                    //todo do we need to update clazz?
                    member_name.unwrap_normal_object().fields.borrow_mut().insert("flags".to_string(), JavaValue::Int(new_flags));
                    if debug{
                        dbg!(&member_name);
                    }
                },
            };
        } else {
            unimplemented!()
        }
    } else {
        unimplemented!();
    }
    JavaValue::Object(member_name.into()).into()
}

pub fn MHN_getConstant() -> Option<JavaValue> {
//todo
    JavaValue::Int(0).into()
}

pub const BRIDGE: i32 = 64;
pub const VARARGS: i32 = 128;
pub const SYNTHETIC: i32 = 4096;
pub const ANNOTATION: i32 = 8192;
pub const ENUM: i32 = 16384;
pub const RECOGNIZED_MODIFIERS: i32 = 65535;
pub const IS_METHOD: i32 = 65536;
pub const IS_CONSTRUCTOR: i32 = 131072;
pub const IS_FIELD: i32 = 262144;
pub const IS_TYPE: i32 = 524288;
pub const CALLER_SENSITIVE: i32 = 1048576;
pub const ALL_ACCESS: i32 = 7;
pub const ALL_KINDS: i32 = 983040;
pub const IS_INVOCABLE: i32 = 196608;
pub const IS_FIELD_OR_METHOD: i32 = 327680;
pub const SEARCH_ALL_SUPERS: i32 = 3145728;
pub const REFERENCE_KIND_SHIFT: u32 = 24;

pub fn MHN_init(state: & JVMState, frame: &StackEntry, args: &mut Vec<JavaValue>) -> Option<JavaValue> {
    //two params, is a static function.
    // init(MemberName mname, Object target);
    let mname = args[0].unwrap_normal_object();
    let target = args[1].unwrap_normal_object();
    // let name = mname.fields.borrow().get("name").unwrap().unwrap_object().map(|x|JavaValue::Object(x.into()).cast_string().to_rust_string());
    let debug = true;//name == "checkSpreadArgument".to_string().into();
    if target.class_pointer.class_view.name() == ClassName::method() {
        let flags = mname.fields.borrow().get("flags").unwrap().unwrap_int();
        let method_fields = target.fields.borrow();
        let clazz = method_fields.get("clazz").unwrap();
        mname.fields.borrow_mut().insert("clazz".to_string(),clazz.clone());
        //todo need to resolve and then indicate the type of call
        //static v. invoke_virtual v. interface
        //see MethodHandles::init_method_MemberName
        let invoke_type_flag = ((if (flags | ACC_STATIC as i32) > 0{
            REF_invokeStatic
        }else {
            let class_ptye = clazz.unwrap_normal_object().class_object_ptype.borrow();
            let class_name = class_ptye.as_ref().unwrap().unwrap_ref_type().try_unwrap_name().unwrap_or_else(|| unimplemented!("Handle arrays?"));
            let inited_class = check_inited_class(state, &class_name, frame.class_pointer.loader.clone());
            if inited_class.class_view.is_interface() {
                REF_invokeInterface
            } else {
                REF_invokeVirtual
            }
        } as u32 ) << REFERENCE_KIND_SHIFT) as i32;
        let extra_flags = IS_METHOD | invoke_type_flag;


        // let signature = method_fields.get("signature").unwrap();


        // dbg!(signature);
        // create_method_type(state,frame,&string_obj_to_string(signature.unwrap_object()));
        // mname.fields.borrow_mut().insert("type".to_string(),frame.pop());

        let modifiers = method_fields.get("modifiers").unwrap().unwrap_int();
        mname.fields.borrow_mut().insert("flags".to_string(),JavaValue::Int(flags | modifiers | extra_flags));//todo is this really correct? what if garbage in flags?
        // let name = method_fields.get("name").unwrap();
        // mname.fields.borrow_mut().insert("name".to_string(),name.clone());
        if debug {
            dbg!(mname);
        }
    } else {

        //todo handle constructors and fields
        unimplemented!()
    }
    None//this is a void method.
}

pub fn create_method_type(state: & JVMState, frame : &StackEntry, signature : &String) {
    //todo should this actually be resolving or is that only for MHN_init. Why is this done in native code anyway
    //todo need to use MethodTypeForm.findForm
    let loader_arc = frame.class_pointer.loader.clone();
    let method_type_class = check_inited_class(state, &ClassName::method_type(), loader_arc.clone());
    push_new_object(state,frame,&method_type_class);
    let this = frame.pop();
    let method_descriptor = parse_method_descriptor(signature).unwrap();
    let rtype = JavaValue::Object(get_or_create_class_object(state,&PTypeView::from_ptype(&method_descriptor.return_type),frame.clone(),loader_arc.clone()).into());

    let ptypes_as_classes: Vec<JavaValue> = method_descriptor.parameter_types.iter().map(|x|{
        let res = get_or_create_class_object(state, &PTypeView::from_ptype(&x), frame.clone(), loader_arc.clone());
        res
    }).map(|x|{
        JavaValue::Object(x.into())
    }).collect();
    let class_type = PTypeView::Ref(ReferenceTypeView::Class(ClassName::class()));
    let ptypes = JavaValue::Object(Arc::new(Array(ArrayObject{ elems: RefCell::new(ptypes_as_classes), elem_type: class_type })).into());
    run_constructor(state, frame, method_type_class, vec![this.clone(),rtype,ptypes], "([Ljava/lang/Class;Ljava/lang/Class;)V".to_string());
    frame.push(this.clone());
    // let method_type_form_class = check_inited_class(state,&ClassName::method_type_form(),loader_arc.clone());
    // run_static_or_virtual(state,frame,&method_type_form_class,"findForm".to_string(),"(Ljava/lang/invoke/MethodType;)Ljava/lang/invoke/MethodTypeForm;".to_string());
    // this.clone().unwrap_normal_object().fields.borrow_mut().insert("form".to_string(),frame.pop());
    // frame.push(this);
}


//todo this should go in some sort of utils
pub fn run_static_or_virtual(state:& JVMState, class: &Arc<RuntimeClass>, method_name: String, desc_str: String ){
    let res_fun = class.classfile.lookup_method(method_name,desc_str);//todo move this into classview
    let (i,m) = res_fun.unwrap();
    let md = MethodDescriptor::from_legacy(m, class.classfile.deref());
    if m.is_static(){
        invoke_static_impl(state,md,class.clone(),i,m)
    }else {
        invoke_virtual_method_i(state,  md,class.clone(),i,m, false);
    }
}