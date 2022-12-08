pub mod entry {
    use jvmti_jni_bindings::jint;
    use rust_jvm_common::compressed_classfile::field_names::FieldName;


    use crate::{JavaValueCommon, JVMState};
    use crate::new_java_values::NewJavaValueHandle;
    use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;

    pub struct Entry<'gc> {
        pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
    }

    impl<'gc> Entry<'gc> {
        pub fn key(&self, jvm: &'gc JVMState<'gc>) -> NewJavaValueHandle<'gc> {
            self.normal_object.get_var_top_level(jvm, FieldName::field_key())
        }

        pub fn value(&self, jvm: &'gc JVMState<'gc>) -> NewJavaValueHandle<'gc> {
            self.normal_object.get_var_top_level(jvm, FieldName::field_value())
        }

        pub fn hash(&self, jvm: &'gc JVMState<'gc>) -> jint {
            self.normal_object.get_var_top_level(jvm, FieldName::field_hash()).unwrap_int_strict()
        }

        pub fn next(&self, jvm: &'gc JVMState<'gc>) -> NewJavaValueHandle<'gc> {
            self.normal_object.get_var_top_level(jvm, FieldName::field_next())
        }
    }
}
