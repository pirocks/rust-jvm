#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use runtime_common::{InterpreterState, StackEntry};
use std::rc::Rc;
use runtime_common::java_values::{JavaValue, NormalObject};
use std::sync::Arc;
use runtime_common::java_values::Object::Object;
use rust_jvm_common::classnames::ClassName;
use std::cell::RefCell;
use crate::interpreter_util::check_inited_class;
use classfile_view::view::ptype_view::PTypeView;
use crate::rust_jni::get_all_methods;
use utils::string_obj_to_string;
use classfile_view::view::HasAccessFlags;

pub fn MHN_resolve(state: &mut InterpreterState, frame: &Rc<StackEntry>, args: &mut Vec<JavaValue>) -> Option<JavaValue> {
//todo
//so as far as I can find this is undocumented.
//so as far as I can figure out we have a method name and a class
//we lookup for a matching method, throw various kinds of exceptions if it doesn't work
// and return a brand new object
//                        dbg!(&args[0]);
//     dbg!(&args[0].unwrap_object().unwrap().lookup_field("clazz"));
    dbg!(&args[1]);
    let member_name = args[0].unwrap_object().unwrap();
    dbg!(member_name.lookup_field("clazz"));
    dbg!(member_name.lookup_field("name"));
    // dbg!(member_name.lookup_field("type"));
    let type_java_value = member_name.lookup_field("type");
    dbg!(&type_java_value.unwrap_normal_object().class_pointer.class_view.name()); // so this is a string before resolution?
    dbg!(member_name.lookup_field("flags"));
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
        class_pointer: check_inited_class(state, &ClassName::object(), frame.clone().into(), frame.class_pointer.loader.clone()),
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
    if is_field{
        assert!(!is_method);
        unimplemented!()
    }else if is_method || is_constructor{
        assert!(!is_field);
        let clazz_field = member_name.lookup_field("clazz");
        let clazz = clazz_field.unwrap_normal_object();
        let clazz_points_to = clazz.class_object_ptype.borrow().as_ref().unwrap().unwrap_class_type();
        let clazz_as_runtime_class = check_inited_class(state, &clazz_points_to, frame.clone().into(), frame.class_pointer.loader.clone());
        let all_methods = get_all_methods(state, frame.clone(), clazz_as_runtime_class);
        let name = string_obj_to_string(member_name.lookup_field("name").unwrap_object());
        let type_ = type_java_value.unwrap_normal_object();
        if type_.class_pointer.class_view.name() == ClassName::method_type() {
            let r_type_class = type_java_value.unwrap_object_nonnull().lookup_field("rtype").unwrap_object_nonnull();
            let param_types_class = type_java_value.unwrap_object_nonnull().lookup_field("ptypes").unwrap_array().unwrap_object_array_nonnull();
            let r_type_as_ptype = r_type_class.unwrap_normal_object().class_object_ptype.borrow().as_ref().unwrap().clone();
            let params_as_ptype: Vec<PTypeView> = param_types_class.iter().map(|x| { x.unwrap_normal_object().class_object_ptype.borrow().as_ref().unwrap().clone() }).collect();

            // private final Class<?> rtype;
            // private final Class<?>[] ptypes;
            let (resolved_method_runtime_class,resolved_i) = all_methods.iter().find(|(x,i)|{
                let c_method  = x.class_view.method_view_i(*i);
                // dbg!(c_method.name());
                // dbg!(&name);
                // dbg!(c_method.desc());
                // dbg!(&r_type_as_ptype);
                // dbg!(&params_as_ptype);
                c_method.name() == name && c_method.desc().return_type == r_type_as_ptype && c_method.desc().parameter_types == params_as_ptype
            }).unwrap();//todo handle not found case
            dbg!(resolved_method_runtime_class.class_view.name());
            dbg!(resolved_i);
            let correct_flags = resolved_method_runtime_class.class_view.method_view_i(*resolved_i).access_flags();
            let new_flags  = (((flags_val as u32) /*& 0xffff*/) | (correct_flags as u32)) as i32;

            //todo do we need to update clazz?
            member_name.unwrap_normal_object().fields.borrow_mut().insert("flags".to_string(), JavaValue::Int(new_flags));
        } else {
            unimplemented!()
        }
    }else {
        unimplemented!();
    }
    JavaValue::Object(member_name.into()).into()
}

pub fn MHN_getConstant() -> Option<JavaValue> {
//todo
    JavaValue::Int(0).into()
}
