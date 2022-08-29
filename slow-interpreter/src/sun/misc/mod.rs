pub mod unsafe_ {
    use std::ops::Deref;
    use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName, MethodName};

    use crate::{InterpreterStateGuard, JVMState, NewAsObjectOrJavaValue, NewJavaValueHandle};
    use crate::class_loading::assert_inited_or_initing_class;
    use another_jit_vm_ir::WasException;
    use crate::java::lang::reflect::field::Field;
    use crate::java_values::{JavaValue};
    use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
    use crate::new_java_values::owned_casts::OwnedCastAble;
    use crate::runtime_class::static_vars;
    use crate::utils::run_static_or_virtual;

    pub struct Unsafe<'gc> {
        pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
    }

    impl<'gc> JavaValue<'gc> {
        pub fn cast_unsafe(&self) -> Unsafe<'gc> {
            Unsafe { normal_object: todo!()/*self.unwrap_object_nonnull()*/ }
        }
    }

    impl<'gc> Unsafe<'gc> {
        pub fn the_unsafe<'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc,'l>) -> Unsafe<'gc> {
            let unsafe_class = assert_inited_or_initing_class(jvm, CClassName::unsafe_().into());
            let static_vars = static_vars(unsafe_class.deref(),jvm);
            static_vars.get(FieldName::field_theUnsafe()).cast_unsafe()
        }

        pub fn object_field_offset<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc,'l>, field: Field<'gc>) -> Result<NewJavaValueHandle<'gc>, WasException> {
            let unsafe_class = assert_inited_or_initing_class(jvm, CClassName::unsafe_().into());
            let desc = CMethodDescriptor { arg_types: vec![CClassName::field().into()], return_type: CPDType::LongType };
            let args = vec![self.normal_object.new_java_value(), field.new_java_value()];
            let res = run_static_or_virtual(jvm, int_state, &unsafe_class, MethodName::method_objectFieldOffset(), &desc, args)?;
            Ok(res.unwrap())
        }

    }
}

pub mod launcher {
    use rust_jvm_common::compressed_classfile::CMethodDescriptor;
    use rust_jvm_common::compressed_classfile::names::{CClassName, MethodName};
    use crate::AllocatedHandle;

    use crate::class_loading::check_initing_or_inited_class;
    use another_jit_vm_ir::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::java::lang::class_loader::ClassLoader;
    use crate::jvm_state::JVMState;
    use crate::new_java_values::{NewJavaValueHandle};
    use crate::utils::run_static_or_virtual;

    pub struct Launcher<'gc> {
        normal_object: AllocatedHandle<'gc>,
    }

    impl<'gc> AllocatedHandle<'gc> {
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
            let launcher = check_initing_or_inited_class(jvm, /*int_state*/todo!(), CClassName::launcher().into())?;
            let res = run_static_or_virtual(jvm, int_state, &launcher, MethodName::method_getLauncher(), &CMethodDescriptor::empty_args(CClassName::launcher().into()), vec![])?.unwrap();
            Ok(res.cast_launcher())
        }

        pub fn get_loader<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc,'l>) -> Result<ClassLoader<'gc>, WasException> {
            let launcher = check_initing_or_inited_class(jvm, /*int_state*/todo!(), CClassName::launcher().into())?;
            let res = run_static_or_virtual(jvm, int_state, &launcher, MethodName::method_getClassLoader(), &CMethodDescriptor::empty_args(CClassName::classloader().into()), vec![self.normal_object.new_java_value()])?.unwrap();
            Ok(res.cast_class_loader())
        }

        //as_object_or_java_value!();
    }

    pub mod ext_class_loader {
        use rust_jvm_common::compressed_classfile::CMethodDescriptor;
        use rust_jvm_common::compressed_classfile::names::{CClassName, MethodName};

        use crate::class_loading::check_initing_or_inited_class;
        use another_jit_vm_ir::WasException;
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
                let ext_class_loader = check_initing_or_inited_class(jvm, /*int_state*/todo!(), CClassName::ext_class_loader().into())?;
                run_static_or_virtual(jvm, int_state, &ext_class_loader, MethodName::method_getExtClassLoader(), &CMethodDescriptor::empty_args(CClassName::launcher().into()), todo!())?;
                Ok(int_state.pop_current_operand_stack(Some(CClassName::classloader().into())).cast_ext_class_launcher())
            }

            //as_object_or_java_value!();
        }
    }
}