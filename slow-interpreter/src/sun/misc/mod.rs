pub mod unsafe_ {
    use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName, MethodName};
    use rust_jvm_common::runtime_type::RuntimeType;

    use crate::{InterpreterStateGuard, JVMState};
    use crate::class_loading::assert_inited_or_initing_class;
    use crate::interpreter::WasException;
    use crate::java::lang::reflect::field::Field;
    use crate::java_values::{GcManagedObject, JavaValue};
    use crate::utils::run_static_or_virtual;

    pub struct Unsafe<'gc_life> {
        normal_object: GcManagedObject<'gc_life>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_unsafe(&self) -> Unsafe<'gc_life> {
            Unsafe { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> Unsafe<'gc_life> {
        pub fn the_unsafe(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>) -> Unsafe<'gc_life> {
            let unsafe_class = assert_inited_or_initing_class(jvm, CClassName::unsafe_().into());
            let static_vars = unsafe_class.static_vars();
            static_vars.get(&FieldName::field_theUnsafe()).unwrap().clone().cast_unsafe()
        }

        pub fn object_field_offset(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, field: Field<'gc_life>) -> Result<JavaValue<'gc_life>, WasException> {
            int_state.push_current_operand_stack(JavaValue::Object(self.normal_object.clone().into()));
            int_state.push_current_operand_stack(field.java_value());
            let rc = self.normal_object.unwrap_normal_object().objinfo.class_pointer.clone();
            let desc = CMethodDescriptor { arg_types: vec![CClassName::field().into()], return_type: CPDType::LongType };
            run_static_or_virtual(jvm, int_state, &rc, MethodName::method_objectFieldOffset(), &desc)?;
            let res = int_state.pop_current_operand_stack(Some(RuntimeType::LongType));
            dbg!(res.to_type());
            dbg!(res.clone());
            Ok(res)
        }

        as_object_or_java_value!();
    }
}

pub mod launcher {
    use rust_jvm_common::compressed_classfile::CMethodDescriptor;
    use rust_jvm_common::compressed_classfile::names::{CClassName, MethodName};

    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::java::lang::class_loader::ClassLoader;
    use crate::java_values::{GcManagedObject, JavaValue};
    use crate::jvm_state::JVMState;
    use crate::utils::run_static_or_virtual;

    pub struct Launcher<'gc_life> {
        normal_object: GcManagedObject<'gc_life>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_launcher(&self) -> Launcher<'gc_life> {
            Launcher { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> Launcher<'gc_life> {
        pub fn get_launcher(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>) -> Result<Launcher<'gc_life>, WasException> {
            let launcher = check_initing_or_inited_class(jvm, int_state, CClassName::launcher().into())?;
            run_static_or_virtual(jvm, int_state, &launcher, MethodName::method_getLauncher(), &CMethodDescriptor::empty_args(CClassName::launcher().into()))?;
            Ok(int_state.pop_current_operand_stack(Some(CClassName::object().into())).cast_launcher())
        }

        pub fn get_loader(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>) -> Result<ClassLoader<'gc_life>, WasException> {
            let launcher = check_initing_or_inited_class(jvm, int_state, CClassName::launcher().into())?;
            int_state.push_current_operand_stack(JavaValue::Object(self.normal_object.clone().into()));
            run_static_or_virtual(jvm, int_state, &launcher, MethodName::method_getClassLoader(), &CMethodDescriptor::empty_args(CClassName::classloader().into()))?;
            Ok(int_state.pop_current_operand_stack(Some(CClassName::classloader().into())).cast_class_loader())
        }

        as_object_or_java_value!();
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

        pub struct ExtClassLoader<'gc_life> {
            normal_object: GcManagedObject<'gc_life>,
        }

        impl<'gc_life> JavaValue<'gc_life> {
            pub fn cast_ext_class_launcher(&self) -> ExtClassLoader<'gc_life> {
                ExtClassLoader { normal_object: self.unwrap_object_nonnull() }
            }
        }

        impl<'gc_life> ExtClassLoader<'gc_life> {
            pub fn get_ext_class_loader(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>) -> Result<ExtClassLoader<'gc_life>, WasException> {
                let ext_class_loader = check_initing_or_inited_class(jvm, int_state, CClassName::ext_class_loader().into())?;
                run_static_or_virtual(jvm, int_state, &ext_class_loader, MethodName::method_getExtClassLoader(), &CMethodDescriptor::empty_args(CClassName::launcher().into()))?;
                Ok(int_state.pop_current_operand_stack(Some(CClassName::classloader().into())).cast_ext_class_launcher())
            }

            as_object_or_java_value!();
        }
    }
}