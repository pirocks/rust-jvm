use crate::{JVMState, StackEntry};
use crate::runtime_class::{prepare_class, RuntimeClass};
use crate::runtime_class::initialize_class;
use std::sync::Arc;
use rust_jvm_common::classnames::ClassName;
use classfile_view::loading::LoaderArc;
use crate::java_values::{JavaValue, default_value, Object};
use descriptor_parser::{parse_method_descriptor};
use crate::instructions::invoke::virtual_::invoke_virtual_method_i;
use classfile_view::view::{HasAccessFlags, ClassView};


//todo jni should really live in interpreter state


// pub mod class_and_object_initialization {
//     use crate::JVMState;
//     use crate::runtime_class::{RuntimeClass, prepare_class, initialize_class};
//     use std::sync::Arc;
//     use rust_jvm_common::classnames::ClassName;
//     use classfile_view::loading::LoaderArc;
//     use crate::instructions::invoke::virtual_::invoke_virtual_method_i;
//     use descriptor_parser::parse_method_descriptor;
//     use crate::stack_entry::StackEntry;
//     use crate::java_values::JavaValue;

    pub fn push_new_object(jvm: &JVMState, current_frame: &StackEntry, target_classfile: &Arc<RuntimeClass>) {
        let loader_arc = &current_frame.class_pointer.loader(jvm).clone();
        let object_pointer = JavaValue::new_object(jvm, target_classfile.clone(), target_classfile.ptypeview());
        let new_obj = JavaValue::Object(object_pointer.clone());
        default_init_fields(jvm, loader_arc.clone(), object_pointer, target_classfile.view(), loader_arc.clone());
        current_frame.push(new_obj);
    }

    fn default_init_fields(jvm: &JVMState, loader_arc: LoaderArc, object_pointer: Option<Arc<Object>>, view: &ClassView, bl: LoaderArc) {
        if view.super_name().is_some() {
            let super_name = view.super_name();
            let loaded_super = loader_arc.load_class(loader_arc.clone(), &super_name.unwrap(), bl.clone(), jvm.get_live_object_pool_getter()).unwrap();
            default_init_fields(jvm, loader_arc.clone(), object_pointer.clone(), &loaded_super, bl);
        }
        for field in view.fields() {
            if !field.is_static() {
                //todo should I look for constant val attributes?
                let _value_i = match field.constant_value_attribute() {
                    None => {}
                    Some(_i) => unimplemented!(),
                };
                let name = field.field_name();
                let type_ = field.field_type();
                let val = default_value(type_);
                {
                    object_pointer.clone().unwrap().unwrap_normal_object().fields.borrow_mut().insert(name, val);
                }
            }
        }
    }

    pub fn run_constructor(state: &JVMState, frame: &StackEntry, target_classfile: Arc<RuntimeClass>, full_args: Vec<JavaValue>, descriptor: String) {
        let method_view = target_classfile.view().method_index().lookup(&"<init>".to_string(), &parse_method_descriptor(descriptor.as_str()).unwrap()).unwrap();
        let md = method_view.desc();
        let this_ptr = full_args[0].clone();
        let actual_args = &full_args[1..];
        frame.push(this_ptr);
        for arg in actual_args {
            frame.push(arg.clone());
        }
        //todo this should be invoke special
        invoke_virtual_method_i(state, md, target_classfile.clone(), method_view.method_i(), &method_view, false);
    }


    pub fn check_inited_class(
        jvm: &JVMState,
        class_name: &ClassName,
        loader_arc: LoaderArc,
    ) -> Arc<RuntimeClass> {
        //todo racy/needs sychronization
        if !jvm.initialized_classes.read().unwrap().contains_key(&class_name) {
            //todo the below is jank
            if class_name == &ClassName::raw_byte() {
                return check_inited_class(jvm, &ClassName::byte(), loader_arc);
            }
            if class_name == &ClassName::raw_char() {
                return check_inited_class(jvm, &ClassName::character(), loader_arc);
            }
            if class_name == &ClassName::raw_double() {
                return check_inited_class(jvm, &ClassName::double(), loader_arc);
            }
            if class_name == &ClassName::raw_float() {
                return check_inited_class(jvm, &ClassName::float(), loader_arc);
            }
            if class_name == &ClassName::raw_int() {
                return check_inited_class(jvm, &ClassName::int(), loader_arc);
            }
            if class_name == &ClassName::raw_long() {
                return check_inited_class(jvm, &ClassName::long(), loader_arc);
            }
            if class_name == &ClassName::raw_short() {
                return check_inited_class(jvm, &ClassName::short(), loader_arc);
            }
            if class_name == &ClassName::raw_boolean() {
                return check_inited_class(jvm, &ClassName::boolean(), loader_arc);
            }
            if class_name == &ClassName::raw_void() {
                return check_inited_class(jvm, &ClassName::void(), loader_arc);
            }
            let bl = jvm.bootstrap_loader.clone();
            jvm.tracing.trace_class_loads(&class_name);
            let target_classfile = loader_arc.clone().load_class(loader_arc.clone(), &class_name, bl, jvm.get_live_object_pool_getter()).unwrap();

            let prepared = Arc::new(prepare_class(target_classfile.backing_class(), loader_arc.clone()));
            jvm.initialized_classes.write().unwrap().insert(class_name.clone(), prepared.clone());//must be before, otherwise infinite recurse
            let inited_target = initialize_class(prepared, jvm);
            jvm.initialized_classes.write().unwrap().insert(class_name.clone(), inited_target);
            jvm.jvmti_state.built_in_jdwp.class_prepare(jvm, class_name)
        }
        let res = jvm.initialized_classes.read().unwrap().get(class_name).unwrap().clone();
        res
    }
// }