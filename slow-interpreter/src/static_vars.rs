use runtime_class_stuff::{RuntimeClass, RuntimeClassClass};
use runtime_class_stuff::accessor::Accessor;
use runtime_class_stuff::field_numbers::FieldNameAndClass;
use rust_jvm_common::compressed_classfile::compressed_types::CPDType;
use rust_jvm_common::compressed_classfile::field_names::FieldName;
use crate::{JavaValueCommon, JVMState, NewJavaValueHandle};
use crate::accessor_ext::AccessorExt;

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

    pub fn get(&self, name: FieldName, expected_type: CPDType) -> NewJavaValueHandle<'gc> {
        let class_name = self.runtime_class_class.class_view.name().unwrap_name();
        //todo need to figure out aliasing

        let static_field = self.jvm.all_the_static_fields.get(FieldNameAndClass { field_name: name, class_name });
        static_field.read_njv(self.jvm, expected_type)
    }

    fn set_raw(&mut self, field_name: FieldName, native: u64) {
        //todo really need static objects layout for all objects
        let class_name = self.runtime_class_class.class_view.name().unwrap_name();
        let static_field = self.jvm.all_the_static_fields.get(FieldNameAndClass { field_name, class_name });
        static_field.write_impl(native)
    }

    pub fn set(&mut self, name: FieldName, elem: NewJavaValueHandle<'gc>) {
        let native_jv = elem.to_stack_native();
        let as_u64 = unsafe { native_jv.as_u64 };//todo this needs to be cleaner
        self.set_raw(name, as_u64);
    }
}
