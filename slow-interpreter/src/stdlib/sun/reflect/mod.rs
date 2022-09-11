pub mod generics;

pub mod reflection {
    use jvmti_jni_bindings::jboolean;
    use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::names::{CClassName, MethodName};

    use crate::{NewAsObjectOrJavaValue, NewJavaValueHandle, PushableFrame, WasException};
    use crate::class_loading::check_initing_or_inited_class;
    use crate::jvm_state::JVMState;
    use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
    use crate::stdlib::java::lang::class::JClass;
    use crate::utils::run_static_or_virtual;

    pub struct Reflection<'gc> {
        normal_object: AllocatedNormalObjectHandle<'gc>,
    }

    impl<'gc> NewJavaValueHandle<'gc> {
        pub fn cast_reflection(self) -> Reflection<'gc> {
            Reflection { normal_object: self.unwrap_object_nonnull().unwrap_normal_object() }
        }
    }

    impl<'gc> Reflection<'gc> {
        pub fn is_same_class_package<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, class1: JClass<'gc>, class2: JClass<'gc>) -> Result<jboolean, WasException<'gc>> {
            let reflection = check_initing_or_inited_class(jvm, int_state, CClassName::reflection().into())?;
            todo!();// int_state.push_current_operand_stack(class1.java_value());
            todo!();// int_state.push_current_operand_stack(class2.java_value()); //I hope these are in the right order, but it shouldn't matter
            let desc = CMethodDescriptor {
                arg_types: vec![CClassName::class().into(), CClassName::class().into()],
                return_type: CPDType::BooleanType,
            };
            run_static_or_virtual(jvm, int_state, &reflection, MethodName::method_isSameClassPackage(), &desc, todo!())?;
            Ok(todo!()/*int_state.pop_current_operand_stack(Some(RuntimeType::IntType)).unwrap_boolean()*/)
        }

    }

    impl<'gc> NewAsObjectOrJavaValue<'gc> for Reflection<'gc> {
        fn object(self) -> AllocatedNormalObjectHandle<'gc> {
            self.normal_object
        }

        fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
            &self.normal_object
        }
    }
}

pub mod constant_pool {
    use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName};

    use crate::{AllocatedHandle, PushableFrame, WasException};
    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter_util::new_object_full;
    use crate::jvm_state::JVMState;
    use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
    use crate::stdlib::java::lang::class::JClass;
    use crate::stdlib::java::NewAsObjectOrJavaValue;

    pub struct ConstantPool<'gc> {
        normal_object: AllocatedNormalObjectHandle<'gc>,
    }

    impl<'gc> AllocatedHandle<'gc> {
        pub fn cast_constant_pool(self) -> ConstantPool<'gc> {
            ConstantPool { normal_object: self.unwrap_normal_object() }
        }
    }

    impl<'gc> ConstantPool<'gc> {
        pub fn new<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, class: JClass<'gc>) -> Result<ConstantPool<'gc>, WasException<'gc>> {
            let constant_pool_classfile = check_initing_or_inited_class(jvm, int_state, CClassName::constant_pool().into())?;
            let constant_pool_object = new_object_full(jvm, int_state, &constant_pool_classfile);
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
    }

    impl<'gc> NewAsObjectOrJavaValue<'gc> for ConstantPool<'gc> {
        fn object(self) -> AllocatedNormalObjectHandle<'gc> {
            self.normal_object
        }

        fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
            &self.normal_object
        }
    }
}