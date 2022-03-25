pub mod unsafe_ {
    use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName, MethodName};
    use rust_jvm_common::runtime_type::RuntimeType;

    use crate::{InterpreterStateGuard, JVMState, NewAsObjectOrJavaValue};
    use crate::class_loading::assert_inited_or_initing_class;
    use crate::interpreter::WasException;
    use crate::java::lang::reflect::field::Field;
    use crate::java_values::{GcManagedObject, JavaValue};
    use crate::utils::run_static_or_virtual;

    pub struct Unsafe<'gc> {
        normal_object: GcManagedObject<'gc>,
    }

    impl<'gc> JavaValue<'gc> {
        pub fn cast_unsafe(&self) -> Unsafe<'gc> {
            Unsafe { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc> Unsafe<'gc> {
        pub fn the_unsafe<'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc,'l>) -> Unsafe<'gc> {
            let unsafe_class = assert_inited_or_initing_class(jvm, CClassName::unsafe_().into());
            let static_vars = unsafe_class.static_vars(jvm);
            static_vars.get(FieldName::field_theUnsafe()).to_jv().cast_unsafe()
        }

        pub fn object_field_offset<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc,'l>, field: Field<'gc>) -> Result<JavaValue<'gc>, WasException> {
            int_state.push_current_operand_stack(JavaValue::Object(self.normal_object.clone().into()));
            int_state.push_current_operand_stack(field.java_value());
            let rc = self.normal_object.unwrap_normal_object().objinfo.class_pointer.clone();
            let desc = CMethodDescriptor { arg_types: vec![CClassName::field().into()], return_type: CPDType::LongType };
            run_static_or_virtual(jvm, int_state, &rc, MethodName::method_objectFieldOffset(), &desc, todo!())?;
            let res = int_state.pop_current_operand_stack(Some(RuntimeType::LongType));
            dbg!(res.to_type());
            dbg!(res.clone());
            Ok(res)
        }

        //as_object_or_java_value!();
    }
}

pub mod launcher {
    use rust_jvm_common::compressed_classfile::CMethodDescriptor;
    use rust_jvm_common::compressed_classfile::names::{CClassName, MethodName};

    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::java::lang::class_loader::ClassLoader;
    use crate::jvm_state::JVMState;
    use crate::new_java_values::{AllocatedObjectHandle, NewJavaValueHandle};
    use crate::utils::run_static_or_virtual;

    pub struct Launcher<'gc> {
        normal_object: AllocatedObjectHandle<'gc>,
    }

    impl<'gc> AllocatedObjectHandle<'gc> {
        pub fn cast_launcher(self) -> Launcher<'gc> {
            Launcher { normal_object: self }
        }
    }

    impl<'gc> NewJavaValueHandle<'gc> {
        pub fn cast_launcher(self) -> Launcher<'gc> {
            Launcher { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc> Launcher<'gc> {
        pub fn get_launcher<'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc,'l>) -> Result<Launcher<'gc>, WasException> {
            let launcher = check_initing_or_inited_class(jvm, int_state, CClassName::launcher().into())?;
            let res = run_static_or_virtual(jvm, int_state, &launcher, MethodName::method_getLauncher(), &CMethodDescriptor::empty_args(CClassName::launcher().into()), vec![])?.unwrap();
            Ok(res.cast_launcher())
        }

        pub fn get_loader<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc,'l>) -> Result<ClassLoader<'gc>, WasException> {
            let launcher = check_initing_or_inited_class(jvm, int_state, CClassName::launcher().into())?;
            let res = run_static_or_virtual(jvm, int_state, &launcher, MethodName::method_getClassLoader(), &CMethodDescriptor::empty_args(CClassName::classloader().into()), vec![self.normal_object.new_java_value()])?.unwrap();
            Ok(res.cast_class_loader())
        }

        //as_object_or_java_value!();
    }

    pub mod ext_class_loader {
        use rust_jvm_common::compressed_classfile::CMethodDescriptor;
        use rust_jvm_common::compressed_classfile::names::{CClassName, MethodName};

        use crate::class_loading::check_initing_or_inited_class;
        use crate::interpreter::WasException;
        use crate::interpreter_state::InterpreterStateGuard;
        use crate::java_values::{GcManagedObject, JavaValue};
        use crate::jvm_state::JVMState;
        use crate::utils::run_static_or_virtual;

        pub struct ExtClassLoader<'gc> {
            normal_object: GcManagedObject<'gc>,
        }

        impl<'gc> JavaValue<'gc> {
            pub fn cast_ext_class_launcher(&self) -> ExtClassLoader<'gc> {
                ExtClassLoader { normal_object: self.unwrap_object_nonnull() }
            }
        }

        impl<'gc> ExtClassLoader<'gc> {
            pub fn get_ext_class_loader<'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc,'l>) -> Result<ExtClassLoader<'gc>, WasException> {
                let ext_class_loader = check_initing_or_inited_class(jvm, int_state, CClassName::ext_class_loader().into())?;
                run_static_or_virtual(jvm, int_state, &ext_class_loader, MethodName::method_getExtClassLoader(), &CMethodDescriptor::empty_args(CClassName::launcher().into()), todo!())?;
                Ok(int_state.pop_current_operand_stack(Some(CClassName::classloader().into())).cast_ext_class_launcher())
            }

            //as_object_or_java_value!();
        }
    }
}