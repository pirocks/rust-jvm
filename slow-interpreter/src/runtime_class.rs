use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, RwLockWriteGuard};

use classfile_view::view::{ClassView, HasAccessFlags};
use runtime_class_stuff::RuntimeClass;
use rust_jvm_common::compressed_classfile::{CPDType};
use rust_jvm_common::compressed_classfile::names::{FieldName, MethodName};
use rust_jvm_common::NativeJavaValue;

use crate::{InterpreterStateGuard, JavaValueCommon, JVMState, MethodResolverImpl, NewJavaValue, NewJavaValueHandle, run_function, StackEntryPush, WasException};
use crate::instructions::ldc::from_constant_pool_entry;
use crate::java_values::{default_value, native_to_new_java_value};

pub fn initialize_class<'gc, 'l>(runtime_class: Arc<RuntimeClass<'gc>>, jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc, 'l>) -> Result<Arc<RuntimeClass<'gc>>, WasException> {
    // assert!(int_state.throw().is_none());
    //todo make sure all superclasses are iniited first
    //todo make sure all interfaces are initted first
    //todo create a extract string which takes index. same for classname
    {
        let view = &runtime_class.view();
        for field in view.fields() {
            if field.is_static() && field.is_final() {
                //todo do I do this for non-static? Should I?
                let constant_info_view = match field.constant_value_attribute() {
                    None => continue,
                    Some(i) => i,
                };
                let constant_value = from_constant_pool_entry(&constant_info_view, jvm, int_state);
                let name = field.field_name();
                static_vars(runtime_class.deref(), jvm).set(name, constant_value);
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
    jvm.java_vm_state.add_method_if_needed(jvm, &MethodResolverImpl { jvm, loader: int_state.current_loader(jvm) }, method_id);


    let new_stack = StackEntryPush::new_java_frame(jvm, runtime_class.clone(), method_i, locals);

    //todo these java frames may have to be converted to native?
    let mut new_function_frame = int_state.push_frame(new_stack);
    return match run_function(jvm, int_state, &mut new_function_frame) {
        Ok(res) => {
            assert!(res.is_none());
            int_state.pop_frame(jvm, new_function_frame, true);
            if !jvm.config.compiled_mode_active {}
            // if int_state.function_return() {
            //     int_state.set_function_return(false);
            Ok(runtime_class)
            // }
            // panic!()
        }
        Err(WasException {}) => {
            int_state.pop_frame(jvm, new_function_frame, false);
            // dbg!(JavaValue::Object(todo!()/*interpreter_state.throw().clone()*/).cast_object().to_string(jvm, interpreter_state).unwrap().unwrap().to_rust_string(jvm));
            int_state.debug_print_stack_trace(jvm);
            Err(WasException)
        }
    };
}

pub fn prepare_class<'vm, 'l, 'k>(jvm: &'vm JVMState<'vm>, int_state: &'_ mut InterpreterStateGuard<'vm, 'l>, classfile: Arc<dyn ClassView>, res: &mut StaticVarGuard<'vm, 'k>) {
    if let Some(jvmti) = jvm.jvmti_state() {
        if let CPDType::Class(cn) = classfile.type_() {
            jvmti.built_in_jdwp.class_prepare(jvm, &cn, int_state)
        }
    }

    for field in classfile.fields() {
        if field.is_static() {
            let val = default_value(&field.field_type());
            res.set(field.field_name(), val);
        }
    }
}


pub fn static_vars<'l, 'gc>(class: &'l RuntimeClass<'gc>, jvm: &'gc JVMState<'gc>) -> StaticVarGuard<'gc, 'l> {
    match class {
        RuntimeClass::Byte => panic!(),
        RuntimeClass::Boolean => panic!(),
        RuntimeClass::Short => panic!(),
        RuntimeClass::Char => panic!(),
        RuntimeClass::Int => panic!(),
        RuntimeClass::Long => panic!(),
        RuntimeClass::Float => panic!(),
        RuntimeClass::Double => panic!(),
        RuntimeClass::Void => panic!(),
        RuntimeClass::Array(_) => panic!(),
        RuntimeClass::Object(o) => {
            StaticVarGuard {
                jvm,
                data_guard: o.static_vars.write().unwrap(),
                types: &o.static_var_types,
            }
        }
        RuntimeClass::Top => panic!(),
    }
}

pub struct StaticVarGuard<'gc, 'l> {
    jvm: &'gc JVMState<'gc>,
    data_guard: RwLockWriteGuard<'l, HashMap<FieldName, NativeJavaValue<'gc>>>,
    types: &'l HashMap<FieldName, CPDType>,
}

impl<'gc, 'l> StaticVarGuard<'gc, 'l> {
    pub fn try_get(&self, name: FieldName) -> Option<NewJavaValueHandle<'gc>> {
        let cpd_type = self.types.get(&name)?;
        let native = *self.data_guard.get(&name)?;
        Some(native_to_new_java_value(native, cpd_type, self.jvm))
    }

    pub fn get(&self, name: FieldName) -> NewJavaValueHandle<'gc> {
        self.try_get(name).unwrap()
    }

    pub fn set(&mut self, name: FieldName, elem: NewJavaValueHandle<'gc>) {
        let cpd_type = self.types.get(&name).unwrap();
        self.data_guard.insert(name, elem.to_native());
    }
}