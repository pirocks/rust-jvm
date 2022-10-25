use runtime_class_stuff::{RuntimeClass, RuntimeClassClass};
use rust_jvm_common::compressed_classfile::field_names::FieldName;
use rust_jvm_common::StackNativeJavaValue;
use crate::{JavaValueCommon, JVMState, NewJavaValueHandle};
use crate::java_values::{native_to_new_java_value_rtype};

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
        RuntimeClass::Object(runtime_class_class) => {
            StaticVarGuard {
                jvm,
                runtime_class_class,
            }
        }
        RuntimeClass::Top => panic!(),
    }
}

pub struct StaticVarGuard<'gc, 'l> {
    jvm: &'gc JVMState<'gc>,
    runtime_class_class: &'l RuntimeClassClass<'gc>,
}

impl<'gc, 'l> StaticVarGuard<'gc, 'l> {
    pub fn try_get(&self, name: FieldName) -> Option<NewJavaValueHandle<'gc>> {
        let cpd_type = self.runtime_class_class.static_field_numbers.get(&name)?;
        let native = unsafe { self.runtime_class_class.static_vars.get(cpd_type.static_number).as_ptr().read() };
        Some(native_to_new_java_value_rtype(StackNativeJavaValue { as_u64: native }, cpd_type.cpdtype.to_runtime_type().unwrap(), self.jvm))
    }

    pub fn get(&self, name: FieldName) -> NewJavaValueHandle<'gc> {
        self.try_get(name).unwrap()
    }

    fn set_raw(&mut self, name: FieldName, native: u64) -> Option<()> {
        let cpd_type = self.runtime_class_class.static_field_numbers.get(&name)?;
        unsafe { self.runtime_class_class.static_vars.get(cpd_type.static_number).as_ptr().write(native); }
        Some(())
    }

    pub fn set(&mut self, name: FieldName, elem: NewJavaValueHandle<'gc>) {
        self.try_set(name, elem).unwrap()
    }

    fn try_set(&mut self, name: FieldName, elem: NewJavaValueHandle<'gc>) -> Option<()> {
        let native_jv = elem.to_stack_native();
        let as_u64 = unsafe { native_jv.as_u64 };//todo this needs to be cleaner
        self.set_raw(name, as_u64)?;
        Some(())
    }
}
