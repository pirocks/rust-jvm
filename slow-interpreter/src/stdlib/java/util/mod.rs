pub mod concurrent;
pub mod hashtable{
    pub mod entry{
        use jvmti_jni_bindings::jint;
        use rust_jvm_common::compressed_classfile::names::FieldName;
        use crate::{JavaValueCommon, JVMState};
        use crate::new_java_values::{ NewJavaValueHandle};
        use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;

        pub struct Entry<'gc> {
            pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
        }

        impl <'gc> Entry<'gc> {
            pub fn key(&self, jvm: &'gc JVMState<'gc>) -> NewJavaValueHandle<'gc> {
                self.normal_object.get_var_top_level(jvm,FieldName::field_key())
            }

            pub fn value(&self, jvm: &'gc JVMState<'gc>) -> NewJavaValueHandle<'gc> {
                self.normal_object.get_var_top_level(jvm,FieldName::field_value())
            }

            pub fn hash(&self, jvm: &'gc JVMState<'gc>) -> jint {
                self.normal_object.get_var_top_level(jvm,FieldName::field_hash()).unwrap_int_strict()
            }

            pub fn next(&self, jvm: &'gc JVMState<'gc>) -> NewJavaValueHandle<'gc> {
                self.normal_object.get_var_top_level(jvm,FieldName::field_next())
            }

        }
    }
}

pub mod properties {
    use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName, MethodName};

    use crate::{JVMState, WasException};
    use crate::class_loading::assert_inited_or_initing_class;

    use crate::better_java_stack::frames::PushableFrame;
    use crate::stdlib::java::lang::string::JString;
    use crate::stdlib::java::NewAsObjectOrJavaValue;
    use crate::java_values::{JavaValue};
    use crate::new_java_values::{NewJavaValueHandle};
    use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
    use crate::utils::run_static_or_virtual;

    pub struct Properties<'gc> {
        pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
    }

    impl<'gc> JavaValue<'gc> {
        pub fn cast_properties(&self) -> Properties<'gc> {
            todo!()
            /*let res = Properties { normal_object: todo!()/*self.unwrap_object_nonnull()*/ };
            assert_eq!(res.normal_object.unwrap_normal_object().objinfo.class_pointer.view().name(), CClassName::properties().into());
            res*/
        }
    }

    impl<'gc> Properties<'gc> {
        pub fn set_property<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, key: JString<'gc>, value: JString<'gc>) -> Result<NewJavaValueHandle<'gc>, WasException<'gc>> {
            let properties_class = assert_inited_or_initing_class(jvm, CClassName::properties().into());
            let args = vec![self.new_java_value(), key.new_java_value(), value.new_java_value()];
            let desc = CMethodDescriptor {
                arg_types: vec![CClassName::string().into(), CClassName::string().into()],
                return_type: CPDType::object(),
            };
            let res = run_static_or_virtual(jvm, int_state, &properties_class, MethodName::method_setProperty(), &desc, args)?;
            Ok(res.unwrap())
        }

        pub fn get_property<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, key: JString<'gc>) -> Result<Option<JString<'gc>>, WasException<'gc>> {
            let properties_class = assert_inited_or_initing_class(jvm, CClassName::properties().into());
            let args = vec![self.new_java_value(), key.new_java_value()];
            let desc = CMethodDescriptor {
                arg_types: vec![CClassName::string().into()],
                return_type: CClassName::string().into(),
            };
            let res = run_static_or_virtual(jvm, int_state, &properties_class, MethodName::method_getProperty(), &desc, args)?;
            Ok(res.unwrap().cast_string())
        }

        pub fn table(&self, jvm: &'gc JVMState<'gc>) -> NewJavaValueHandle<'gc> {
            let hashtable_rc = assert_inited_or_initing_class(jvm, CClassName::hashtable().into());
            self.normal_object.get_var(jvm,&hashtable_rc, FieldName::field_table())
        }

        /*pub fn map(&self, jvm: &'gc JVMState<'gc>) -> Option<ConcurrentHashMap<'gc>> {
            self.normal_object.as_allocated_obj().get_var_top_level(jvm, FieldName::field_map()).cast_concurrent_hash_map()
        }*/
    }


    impl<'gc> NewAsObjectOrJavaValue<'gc> for Properties<'gc> {
        fn object(self) -> AllocatedNormalObjectHandle<'gc> {
            self.normal_object
        }

        fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
            &self.normal_object
        }
    }
}