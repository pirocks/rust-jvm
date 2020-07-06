pub mod method_type {
    use crate::java_values::{JavaValue, Object};
    use std::sync::Arc;
    use crate::interpreter_util::check_inited_class;
    use rust_jvm_common::classnames::ClassName;
    use crate::java::lang::class_loader::ClassLoader;
    use crate::{JVMState, InterpreterStateGuard};

    pub struct MethodType {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_method_type(&self) -> MethodType {
            MethodType { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl MethodType {
        as_object_or_java_value!();

        pub fn from_method_descriptor_string<'l>(jvm: &'static JVMState, int_state: & mut InterpreterStateGuard, str: crate::java::lang::string::JString, class_loader: Option<ClassLoader>) -> MethodType {
            int_state.push_current_operand_stack(str.java_value());
            int_state.push_current_operand_stack(class_loader.map(|x| x.java_value()).unwrap_or(JavaValue::Object(None)));
            let method_type = check_inited_class(jvm, int_state,&ClassName::method_type().into(), int_state.current_loader(jvm).clone());
            crate::instructions::invoke::native::mhn_temp::run_static_or_virtual(jvm, int_state,&method_type, "fromMethodDescriptorString".to_string(), "(Ljava/lang/String;Ljava/lang/ClassLoader;)Ljava/lang/invoke/MethodType;".to_string());
            int_state.pop_current_operand_stack().cast_method_type()
        }
    }
}


pub mod method_handle {
    use crate::java_values::{JavaValue, Object};
    use crate::{JVMState, InterpreterStateGuard};

    use crate::java::lang::string::JString;
    use crate::instructions::invoke::native::mhn_temp::run_static_or_virtual;
    use std::sync::Arc;
    use crate::java::lang::invoke::method_type::MethodType;
    use crate::java::lang::class::JClass;
    use crate::interpreter_util::check_inited_class;
    use rust_jvm_common::classnames::ClassName;

    pub struct MethodHandle {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_method_handle(&self) -> MethodHandle {
            MethodHandle { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl MethodHandle {
        pub fn lookup<'l>(jvm: &'static JVMState, int_state: & mut InterpreterStateGuard) -> Lookup {
            let method_handles_class = check_inited_class(jvm, int_state,&ClassName::method_handles().into(), int_state.current_loader(jvm));
            run_static_or_virtual(jvm, int_state,&method_handles_class, "lookup".to_string(), "()Ljava/lang/invoke/MethodHandles$Lookup;".to_string());
            int_state.pop_current_operand_stack().cast_lookup()
        }
        pub fn public_lookup<'l>(jvm: &'static JVMState, int_state: & mut InterpreterStateGuard) -> Lookup {
            let method_handles_class = check_inited_class(jvm, int_state,&ClassName::method_handles().into(), int_state.current_loader(jvm));
            run_static_or_virtual(jvm, int_state,&method_handles_class, "publicLookup".to_string(), "()Ljava/lang/invoke/MethodHandles$Lookup;".to_string());
            int_state.pop_current_operand_stack().cast_lookup()
        }

        as_object_or_java_value!();
    }

    #[derive(Clone)]
    pub struct Lookup {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_lookup(&self) -> Lookup {
            Lookup { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl Lookup {
        pub fn find_virtual<'l>(&self, jvm: &'static JVMState, int_state: & mut InterpreterStateGuard, obj: JClass, name: JString, mt: MethodType) -> MethodHandle {
            let lookup_class = check_inited_class(jvm, int_state,&ClassName::lookup().into(), int_state.current_loader(jvm).clone());
            int_state.push_current_operand_stack(self.clone().java_value());
            int_state.push_current_operand_stack(obj.java_value());
            int_state.push_current_operand_stack(name.java_value());
            int_state.push_current_operand_stack(mt.java_value());
            run_static_or_virtual(jvm, int_state,&lookup_class, "findVirtual".to_string(), "(Ljava/lang/Class;Ljava/lang/String;Ljava/lang/invoke/MethodType;)Ljava/lang/invoke/MethodHandle;".to_string());
            int_state.pop_current_operand_stack().cast_method_handle()
        }

        as_object_or_java_value!();
    }
}
