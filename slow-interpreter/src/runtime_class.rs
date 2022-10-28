use std::ops::Deref;
use std::sync::Arc;

use classfile_view::view::{ClassView, HasAccessFlags};
use runtime_class_stuff::{RuntimeClass};
use rust_jvm_common::compressed_classfile::compressed_types::CPDType;
use rust_jvm_common::compressed_classfile::method_names::MethodName;


use crate::{JVMState, MethodResolverImpl, NewJavaValue, run_function, StackEntryPush, WasException};
use crate::better_java_stack::frames::PushableFrame;
use crate::interpreter::common::ldc::from_constant_pool_entry;
use crate::java_values::{default_value};
use crate::static_vars::{static_vars, StaticVarGuard};

pub fn initialize_class<'gc, 'l>(runtime_class: Arc<RuntimeClass<'gc>>, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<Arc<RuntimeClass<'gc>>, WasException<'gc>> {
    // assert!(int_state.throw().is_none());
    //todo make sure all superclasses are iniited first
    //todo make sure all interfaces are initted first
    //todo create a extract string which takes index. same for classname
    {
        let view = &runtime_class.view();
        for field in view.fields() {
            if field.is_static() && field.is_final() {
                //todo do I do this for non-static? Should I?
                let field_name = field.field_name();
                match field.constant_value_attribute() {
                    None => {
                        static_vars(runtime_class.deref(), jvm).set(field_name, default_value(field.field_type()));
                    },
                    Some(constant_info_view) => {
                        let constant_value = from_constant_pool_entry(&constant_info_view, jvm, int_state);
                        static_vars(runtime_class.deref(), jvm).set(field_name, constant_value);
                    },
                }
            }
        }
    }
    //todo detecting if assertions are enabled?
    let view = &runtime_class.view();
    let lookup_res = view.lookup_method_name(MethodName::constructor_clinit()); // todo constant for clinit
    assert!(lookup_res.len() <= 1);
    let clinit = match lookup_res.get(0) {
        None => return Ok(runtime_class),
        Some(x) => x,
    };
    //todo should I really be manipulating the interpreter state like this

    let mut locals = vec![];
    let locals_n = clinit.code_attribute().unwrap().max_locals;
    for _ in 0..locals_n {
        locals.push(NewJavaValue::Top);
    }

    let method_i = clinit.method_i() as u16;
    let method_id = jvm.method_table.write().unwrap().get_method_id(runtime_class.clone(), method_i);
    jvm.java_vm_state.add_method_if_needed(jvm, &MethodResolverImpl { jvm, loader: int_state.current_loader(jvm) }, method_id, false);


    let new_frame = StackEntryPush::new_java_frame(jvm, runtime_class.clone(), method_i, locals);

    //todo these java frames may have to be converted to native?
    // let new_function_frame = int_state.push_frame(new_stack);
    int_state.push_frame_java(new_frame, |java_stack_guard| {
        let res = run_function(jvm, java_stack_guard)?;
        assert!(res.is_none());
        if !jvm.config.compiled_mode_active {}
        Ok(runtime_class)
    })
}

pub fn prepare_class<'vm, 'l, 'k>(jvm: &'vm JVMState<'vm>, int_state: &mut impl PushableFrame<'vm>, class_view: Arc<dyn ClassView>, res: &mut StaticVarGuard<'vm, 'k>) {
    if let Some(jvmti) = jvm.jvmti_state() {
        if let CPDType::Class(cn) = class_view.type_() {
            jvmti.built_in_jdwp.class_prepare(jvm, &cn, int_state)
        }
    }

    for field in class_view.fields() {
        if field.is_static() {
            let val = default_value(field.field_type());
            res.set(field.field_name(), val);
        }
    }
}

