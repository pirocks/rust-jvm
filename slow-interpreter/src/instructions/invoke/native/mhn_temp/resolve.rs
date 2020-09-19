use std::cell::RefCell;
use std::sync::Arc;

use classfile_view::view::HasAccessFlags;
use classfile_view::view::ptype_view::PTypeView;
use rust_jvm_common::classnames::ClassName;

use crate::{InterpreterStateGuard, JVMState};
use crate::instructions::invoke::Object;
use crate::interpreter_util::check_inited_class;
use crate::java_values::{JavaValue, NormalObject};
use crate::rust_jni::interface::misc::{get_all_fields, get_all_methods};
use crate::utils::string_obj_to_string;

pub fn MHN_resolve<'l>(jvm: &JVMState, int_state: &mut InterpreterStateGuard, args: &mut Vec<JavaValue>) -> Option<JavaValue> {
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
    resolve_impl(jvm, int_state, &member_name)
}

fn resolve_impl(jvm: &JVMState, int_state: &mut InterpreterStateGuard, member_name: &Arc<Object>) -> Option<JavaValue> {
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


    let resolution_object = JavaValue::Object(Arc::new(Object::Object(NormalObject {
        monitor: jvm.thread_state.new_monitor("monitor for a resolution object".to_string()),
        fields: RefCell::new(Default::default()),
        class_pointer: check_inited_class(jvm, int_state, &ClassName::object().into(), int_state.current_loader(jvm)),
        class_object_type: None,
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
    let clazz = member_name.lookup_field("clazz").cast_class();
    let clazz_as_runtime_class = clazz.as_runtime_class();
    let name = string_obj_to_string(member_name.lookup_field("name").unwrap_object());
    let debug = &name == "checkSpreadArgument";
    let type_ = type_java_value.unwrap_normal_object();
    if is_field {
        assert!(!is_method);
        let all_fields = get_all_fields(jvm, int_state, clazz_as_runtime_class);
        dbg!(&type_);
        if type_.class_pointer.view().name() == ClassName::class() {
            let typejclass = type_java_value.cast_class();
            let target_ptype = typejclass.as_type();
            let (res_c, res_i) = all_fields.iter().find(|(c, i)| {
                let field = c.view().field(*i);
                field.field_name() == name &&
                    field.field_type() == target_ptype
            }).unwrap();

            let correct_flags = res_c.view().field(*res_i).access_flags();
            let new_flags = ((flags_val as u32) | (correct_flags as u32)) as i32;

            //todo do we need to update clazz?
            member_name.unwrap_normal_object().fields.borrow_mut().insert("flags".to_string(), JavaValue::Int(new_flags));
        } else {
            unimplemented!()
        }
    } else if is_method || is_constructor {
        assert!(!is_constructor);//todo not implemented yet
        assert!(!is_field);
        // frame.print_stack_trace();
        let all_methods = get_all_methods(jvm, int_state, clazz_as_runtime_class);
        if type_.class_pointer.view().name() == ClassName::method_type() {
            let r_type_class = type_java_value.unwrap_object_nonnull().lookup_field("rtype").unwrap_object_nonnull();
            let param_types_class = type_java_value.unwrap_object_nonnull().lookup_field("ptypes").unwrap_array().unwrap_object_array_nonnull();
            let _r_type_as_ptype = JavaValue::Object(r_type_class.into()).cast_class().as_type();
            let params_as_ptype: Vec<PTypeView> = param_types_class.iter().map(|x| {
                JavaValue::Object(x.clone().into()).cast_class().as_type()
            }).collect();
            //todo how do the params work with static v. not static
            match all_methods.iter().find(|(x, i)| {
                let c_method = x.view().method_view_i(*i);
                //todo need to handle signature polymorphism here and in many places
                c_method.name() == name && if c_method.is_signature_polymorphic() {
                    c_method.desc().parameter_types.len() == 1 &&
                        c_method.desc().parameter_types[0] == PTypeView::array(PTypeView::object()).to_ptype() &&
                        c_method.desc().return_type == PTypeView::object().to_ptype()
                } else {
                    c_method.desc().parameter_types == params_as_ptype.iter().map(|x| x.to_ptype()).collect::<Vec<_>>() //todo what about overloading
                }
            }) {
                None => {
                    member_name.unwrap_normal_object().fields.borrow_mut().insert("resolution".to_string(), JavaValue::Object(None));
                }
                Some((resolved_method_runtime_class, resolved_i)) => {
                    let correct_flags = resolved_method_runtime_class.view().method_view_i(*resolved_i).access_flags();
                    let new_flags = ((flags_val as u32) | (correct_flags as u32)) as i32;

                    //todo do we need to update clazz?
                    member_name.unwrap_normal_object().fields.borrow_mut().insert("flags".to_string(), JavaValue::Int(new_flags));
                    if debug {
                        dbg!(&member_name);
                    }
                }
            };
        } else {
            unimplemented!()
        }
    } else {
        unimplemented!();
    }
    JavaValue::Object(member_name.clone().into()).into()
}

pub mod tests {
    use crate::java::lang::class::JClass;
    use crate::java::lang::invoke::method_type::MethodType;
    use crate::java::lang::invoke::method_type_form::MethodTypeForm;
    use crate::java::lang::member_name::MemberName;
    use crate::java::lang::string::JString;

    use super::*;

    pub fn run_tests(jvm: &JVMState, int_state: &mut InterpreterStateGuard) {
        unimplemented!()
    }

    fn call_resolve(jvm: &JVMState, int_state: &mut InterpreterStateGuard, m: MemberName, lookupClass: Option<JClass>) -> JavaValue {
        let lookupClassJavaValue = match lookupClass {
            None => JavaValue::Object(None),
            Some(jclass) => jclass.java_value(),
        };
        MHN_resolve(jvm, int_state, &mut vec![m.java_value(), lookupClassJavaValue]).unwrap()
    }


    fn zero_L_test(jvm: &JVMState, int_state: &mut InterpreterStateGuard) {
        let lambda_form_class = JClass::from_name(jvm, int_state, ClassName::Str("java/lang/invoke/LambdaForm".to_string()));
        let name = JString::from_rust(jvm, int_state, "zero_L".to_string());

        let type_ = {
            let form = {
                let arg_to_slot_table = JavaValue::new_vec_from_vec(jvm, vec![JavaValue::Int(0)], PTypeView::IntType);
                let slot_to_arg_table = JavaValue::new_vec_from_vec(jvm, vec![JavaValue::Int(0)], PTypeView::IntType);
                let method_handles = JavaValue::new_vec_from_vec(jvm, vec![JavaValue::null(); 3], ClassName::Str("java/lang/ref/SoftReference".to_string()).into());
                let lambda_forms = JavaValue::new_vec_from_vec(jvm, vec![JavaValue::null(); 18], ClassName::Str("java/lang/ref/SoftReference".to_string()).into());
                MethodTypeForm::new(
                    jvm,
                    int_state,
                    arg_to_slot_table,
                    slot_to_arg_table,
                    281479271677952,
                    0,
                    None,
                    None,
                    method_handles,
                    lambda_forms,
                )
            };
            let rtype = JClass::from_name(jvm, int_state, ClassName::object());
            MethodType::new(jvm, int_state, rtype, vec![], form, JavaValue::null(), JavaValue::null(), JavaValue::null())
        };
        type_.get_form().set_erased_type(type_.clone());
        type_.get_form().set_basic_type(type_.clone());
        let resolution = MemberName::new_self_resolution(jvm, int_state, lambda_form_class.clone(), name.clone(), type_.clone(), 100728832);
        let member_name = MemberName::new_member_name(jvm, int_state, lambda_form_class, name, type_, 100728832, resolution.java_value());
        let lookupClass = JavaValue::null();
        let mut args = vec![member_name.java_value(), lookupClass];
        MHN_resolve(jvm, int_state, &mut args);
        assert_eq!(member_name.get_flags(), 100728842);
    }


    fn identity_L_test(jvm: &JVMState, int_state: &mut InterpreterStateGuard) {
        let lambda_form_class = JClass::from_name(jvm, int_state, ClassName::Str("java/lang/invoke/LambdaForm".to_string()));
        let name = JString::from_rust(jvm, int_state, "identity_L".to_string());

        let type_ = {
            let form = {
                let arg_to_slot_table = JavaValue::new_vec_from_vec(jvm, vec![JavaValue::Int(0)], PTypeView::IntType);
                let slot_to_arg_table = JavaValue::new_vec_from_vec(jvm, vec![JavaValue::Int(0)], PTypeView::IntType);
                let method_handles = JavaValue::new_vec_from_vec(jvm, vec![JavaValue::null(); 3], ClassName::Str("java/lang/ref/SoftReference".to_string()).into());
                let lambda_forms = JavaValue::new_vec_from_vec(jvm, vec![JavaValue::null(); 18], ClassName::Str("java/lang/ref/SoftReference".to_string()).into());
                MethodTypeForm::new(
                    jvm,
                    int_state,
                    arg_to_slot_table,
                    slot_to_arg_table,
                    281479271677952,
                    0,
                    None,
                    None,
                    method_handles,
                    lambda_forms,
                )
            };
            let rtype = JClass::from_name(jvm, int_state, ClassName::object());
            let ptypes = vec![JClass::from_name(jvm, int_state, ClassName::object())];
            MethodType::new(jvm, int_state, rtype, ptypes, form, JavaValue::null(), JavaValue::null(), JavaValue::null())
        };
        type_.get_form().set_erased_type(type_.clone());
        type_.get_form().set_basic_type(type_.clone());
        let resolution = MemberName::new_self_resolution(jvm, int_state, lambda_form_class.clone(), name.clone(), type_.clone(), 100728832);
        let member_name = MemberName::new_member_name(jvm, int_state, lambda_form_class, name, type_, 100728832, resolution.java_value());
        let lookupClass = JavaValue::null();
        let mut args = vec![member_name.java_value(), lookupClass];
        MHN_resolve(jvm, int_state, &mut args);
        assert_eq!(member_name.get_flags(), 100728842);
    }
}