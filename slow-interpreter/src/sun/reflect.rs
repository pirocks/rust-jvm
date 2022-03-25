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
        pub fn is_same_class_package<'l>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, class1: JClass<'gc_life>, class2: JClass<'gc_life>) -> Result<jboolean, WasException> {
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

pub mod constant_pool {
    use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName};

    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::interpreter_util::new_object;
    use crate::java::lang::class::JClass;
    use crate::java::NewAsObjectOrJavaValue;
    use crate::jvm_state::JVMState;
    use crate::new_java_values::{AllocatedObject, AllocatedObjectHandle};

    pub struct ConstantPool<'gc_life> {
        normal_object: AllocatedObjectHandle<'gc_life>,
    }

    impl<'gc_life> AllocatedObjectHandle<'gc_life> {
        pub fn cast_constant_pool(self) -> ConstantPool<'gc_life> {
            ConstantPool { normal_object: self }
        }
    }

    impl<'gc_life> ConstantPool<'gc_life> {
        pub fn new<'l>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, class: JClass<'gc_life>) -> Result<ConstantPool<'gc_life>, WasException> {
            let constant_pool_classfile = check_initing_or_inited_class(jvm, int_state, CClassName::constant_pool().into())?;
            let constant_pool_object = new_object(jvm, int_state, &constant_pool_classfile);
            let res = constant_pool_object.cast_constant_pool();
            res.set_constant_pool_oop(jvm, class);
            Ok(res)
        }

        pub fn get_constant_pool_oop(&self, jvm: &'gc_life JVMState<'gc_life>) -> JClass<'gc_life> {
            self.normal_object.as_allocated_obj().get_var_top_level(jvm, FieldName::field_constantPoolOop()).cast_class().unwrap()
        }

        pub fn set_constant_pool_oop(&self, jvm: &'gc_life JVMState<'gc_life>, jclass: JClass<'gc_life>) {
            self.normal_object.as_allocated_obj().set_var_top_level(jvm, FieldName::field_constantPoolOop(), jclass.new_java_value());
        }

        // as_object_or_java_value!();
    }

    impl <'gc_life> NewAsObjectOrJavaValue<'gc_life> for ConstantPool<'gc_life>{
        fn object(self) -> AllocatedObjectHandle<'gc_life> {
            self.normal_object
        }

        fn object_ref(&self) -> AllocatedObject<'gc_life, '_> {
            self.normal_object.as_allocated_obj()
        }
    }
}