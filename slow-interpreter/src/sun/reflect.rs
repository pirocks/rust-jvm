pub mod reflection {
    use jvmti_jni_bindings::jboolean;
    use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::names::{CClassName, MethodName};
    use rust_jvm_common::runtime_type::RuntimeType;

    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::java::lang::class::JClass;
    use crate::java::NewAsObjectOrJavaValue;
    use crate::java_values::{GcManagedObject, JavaValue};
    use crate::jvm_state::JVMState;
    use crate::utils::run_static_or_virtual;

    pub struct Reflection<'gc_life> {
        normal_object: GcManagedObject<'gc_life>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_reflection(&self) -> Reflection<'gc_life> {
            Reflection { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> Reflection<'gc_life> {
        pub fn is_same_class_package(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life,'l>, class1: JClass<'gc_life>, class2: JClass<'gc_life>) -> Result<jboolean, WasException> {
            let reflection = check_initing_or_inited_class(jvm, int_state, CClassName::reflection().into())?;
            int_state.push_current_operand_stack(class1.java_value());
            int_state.push_current_operand_stack(class2.java_value()); //I hope these are in the right order, but it shouldn't matter
            let desc = CMethodDescriptor {
                arg_types: vec![CClassName::class().into(), CClassName::class().into()],
                return_type: CPDType::BooleanType,
            };
            run_static_or_virtual(jvm, int_state, &reflection, MethodName::method_isSameClassPackage(), &desc, todo!())?;
            Ok(int_state.pop_current_operand_stack(Some(RuntimeType::IntType)).unwrap_boolean())
        }

        as_object_or_java_value!();
    }
}