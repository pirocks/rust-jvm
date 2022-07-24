pub mod tree{
    pub mod class_signature{
        use itertools::Itertools;
        use rust_jvm_common::compressed_classfile::names::FieldName;
        use crate::{AllocatedHandle, JVMState, NewAsObjectOrJavaValue};
        use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;

        pub struct ClassSignature<'gc> {
            pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
        }

        impl<'gc> ClassSignature<'gc> {
            pub fn get_formal_type_params(&self, jvm: &'gc JVMState<'gc>) -> Vec<Option<AllocatedHandle<'gc>>> {
                self
                    .normal_object
                    .get_var_top_level(jvm, FieldName::field_formalTypeParams())
                    .unwrap_object()
                    .unwrap()
                    .unwrap_array()
                    .array_iterator()
                    .map(|njvh|njvh.unwrap_object())
                    .collect_vec()
            }
        }

        impl<'gc> NewAsObjectOrJavaValue<'gc> for ClassSignature<'gc> {
            fn object(self) -> AllocatedNormalObjectHandle<'gc> {
                self.normal_object
            }

            fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
                &self.normal_object
            }
        }
    }
}

