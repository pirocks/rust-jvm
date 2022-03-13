pub mod concurrent;
pub mod hashtable{
    pub mod entry{
        use jvmti_jni_bindings::jint;
        use rust_jvm_common::compressed_classfile::names::FieldName;
        use crate::JVMState;
        use crate::new_java_values::{AllocatedObjectHandle, NewJavaValueHandle};

        pub struct Entry<'gc_life> {
            normal_object: AllocatedObjectHandle<'gc_life>,
        }

        impl<'gc_life> AllocatedObjectHandle<'gc_life> {
            pub fn cast_entry(self) -> Entry<'gc_life> {
                Entry { normal_object: self }
            }
        }

        impl <'gc_life> Entry<'gc_life> {
            pub fn key(&self, jvm: &'gc_life JVMState<'gc_life>) -> NewJavaValueHandle<'gc_life> {
                self.normal_object.as_allocated_obj().get_var_top_level(jvm,FieldName::field_key())
            }

            pub fn value(&self, jvm: &'gc_life JVMState<'gc_life>) -> NewJavaValueHandle<'gc_life> {
                self.normal_object.as_allocated_obj().get_var_top_level(jvm,FieldName::field_value())
            }

            pub fn hash(&self, jvm: &'gc_life JVMState<'gc_life>) -> jint {
                self.normal_object.as_allocated_obj().get_var_top_level(jvm,FieldName::field_hash()).as_njv().unwrap_int_strict()
            }

            pub fn next(&self, jvm: &'gc_life JVMState<'gc_life>) -> NewJavaValueHandle<'gc_life> {
                self.normal_object.as_allocated_obj().get_var_top_level(jvm,FieldName::field_next())
            }

        }
    }
}

pub mod properties {
    use std::ptr::hash;
    use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName, MethodName};

    use crate::{InterpreterStateGuard, JVMState, NewJavaValue};
    use crate::class_loading::assert_inited_or_initing_class;
    use crate::interpreter::WasException;
    use crate::java::lang::string::JString;
    use crate::java::NewAsObjectOrJavaValue;
    use crate::java::util::concurrent::concurrent_hash_map::ConcurrentHashMap;
    use crate::java_values::{GcManagedObject, JavaValue};
    use crate::new_java_values::{AllocatedObjectHandle, NewJavaValueHandle};
    use crate::utils::run_static_or_virtual;

    pub struct Properties<'gc_life> {
        normal_object: AllocatedObjectHandle<'gc_life>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_properties(&self) -> Properties<'gc_life> {
            todo!()
            /*let res = Properties { normal_object: todo!()/*self.unwrap_object_nonnull()*/ };
            assert_eq!(res.normal_object.unwrap_normal_object().objinfo.class_pointer.view().name(), CClassName::properties().into());
            res*/
        }
    }

    impl<'gc_life> AllocatedObjectHandle<'gc_life> {
        pub fn cast_properties(self) -> Properties<'gc_life> {
            Properties { normal_object: self }
        }
    }

    impl<'gc_life> Properties<'gc_life> {
        pub fn set_property<'l>(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, key: JString<'gc_life>, value: JString<'gc_life>) -> Result<NewJavaValueHandle<'gc_life>, WasException> {
            let properties_class = assert_inited_or_initing_class(jvm, CClassName::properties().into());
            let args = vec![NewJavaValue::AllocObject(self.normal_object.as_allocated_obj()), key.new_java_value(), value.new_java_value()];
            let desc = CMethodDescriptor {
                arg_types: vec![CClassName::string().into(), CClassName::string().into()],
                return_type: CPDType::object(),
            };
            let res = run_static_or_virtual(jvm, int_state, &properties_class, MethodName::method_setProperty(), &desc, args)?;
            Ok(res.unwrap())
        }

        pub fn get_property<'l>(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, key: JString<'gc_life>) -> Result<Option<JString<'gc_life>>, WasException> {
            let properties_class = assert_inited_or_initing_class(jvm, CClassName::properties().into());
            let args = vec![NewJavaValue::AllocObject(self.normal_object.as_allocated_obj()), key.new_java_value()];
            let desc = CMethodDescriptor {
                arg_types: vec![CClassName::string().into()],
                return_type: CClassName::string().into(),
            };
            let res = run_static_or_virtual(jvm, int_state, &properties_class, MethodName::method_getProperty(), &desc, args)?;
            Ok(res.unwrap().cast_string())
        }

        pub fn table(&self, jvm: &'gc_life JVMState<'gc_life>) -> NewJavaValueHandle<'gc_life> {
            let hashtable_rc = assert_inited_or_initing_class(jvm, CClassName::hashtable().into());
            self.normal_object.as_allocated_obj().get_var(jvm,&hashtable_rc, FieldName::field_table())
        }

        /*pub fn map(&self, jvm: &'gc_life JVMState<'gc_life>) -> Option<ConcurrentHashMap<'gc_life>> {
            self.normal_object.as_allocated_obj().get_var_top_level(jvm, FieldName::field_map()).cast_concurrent_hash_map()
        }*/
    }
}