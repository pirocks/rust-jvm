pub mod generics;

pub mod reflection {
    use jvmti_jni_bindings::jboolean;
    use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::names::{CClassName, MethodName};
    use rust_jvm_common::runtime_type::RuntimeType;

    use crate::class_loading::check_initing_or_inited_class;
    use another_jit_vm_ir::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::java::lang::class::JClass;
    use crate::java::NewAsObjectOrJavaValue;
    use crate::java_values::{GcManagedObject, JavaValue};
    use crate::jvm_state::JVMState;
    use crate::utils::run_static_or_virtual;

    pub struct Reflection<'gc> {
        normal_object: GcManagedObject<'gc>,
    }

    impl<'gc> JavaValue<'gc> {
        pub fn cast_reflection(&self) -> Reflection<'gc> {
            Reflection { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc> Reflection<'gc> {
        pub fn is_same_class_package<'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc, 'l>, class1: JClass<'gc>, class2: JClass<'gc>) -> Result<jboolean, WasException> {
            let reflection = check_initing_or_inited_class(jvm, /*int_state*/todo!(), CClassName::reflection().into())?;
            int_state.push_current_operand_stack(class1.java_value());
            int_state.push_current_operand_stack(class2.java_value()); //I hope these are in the right order, but it shouldn't matter
            let desc = CMethodDescriptor {
                arg_types: vec![CClassName::class().into(), CClassName::class().into()],
                return_type: CPDType::BooleanType,
            };
            run_static_or_virtual(jvm, int_state, &reflection, MethodName::method_isSameClassPackage(), &desc, todo!())?;
            Ok(int_state.pop_current_operand_stack(Some(RuntimeType::IntType)).unwrap_boolean())
        }

        //as_object_or_java_value!();
    }
}

pub mod constant_pool {
    use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName};
    use crate::AllocatedHandle;

    use crate::class_loading::check_initing_or_inited_class;
    use another_jit_vm_ir::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::interpreter_util::{new_object_full};
    use crate::java::lang::class::JClass;
    use crate::java::NewAsObjectOrJavaValue;
    use crate::jvm_state::JVMState;

    pub struct ConstantPool<'gc> {
        normal_object: AllocatedNormalObjectHandle<'gc>,
    }

    impl<'gc> AllocatedHandle<'gc> {
        pub fn cast_constant_pool(self) -> ConstantPool<'gc> {
            ConstantPool { normal_object: self.unwrap_normal_object() }
        }
    }

    impl<'gc> ConstantPool<'gc> {
        pub fn new<'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc, 'l>, class: JClass<'gc>) -> Result<ConstantPool<'gc>, WasException> {
            let constant_pool_classfile = check_initing_or_inited_class(jvm, /*int_state*/todo!(), CClassName::constant_pool().into())?;
            let constant_pool_object = new_object_full(jvm, todo!()/*int_state*/, &constant_pool_classfile);
            let res = constant_pool_object.cast_constant_pool();
            res.set_constant_pool_oop(jvm, class);
            Ok(res)
        }

        pub fn get_constant_pool_oop(&self, jvm: &'gc JVMState<'gc>) -> JClass<'gc> {
            self.normal_object.get_var_top_level(jvm, FieldName::field_constantPoolOop()).cast_class().unwrap()
        }

        pub fn set_constant_pool_oop(&self, jvm: &'gc JVMState<'gc>, jclass: JClass<'gc>) {
            self.normal_object.set_var_top_level(jvm, FieldName::field_constantPoolOop(), jclass.new_java_value());
        }

        // as_object_or_java_value!();
    }

    use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
    impl<'gc> NewAsObjectOrJavaValue<'gc> for ConstantPool<'gc> {
        fn object(self) -> AllocatedNormalObjectHandle<'gc> {
            self.normal_object
        }

        fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
            &self.normal_object
        }
    }
}