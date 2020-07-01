pub mod method_type {
    use crate::java_values::{JavaValue, Object};
    use std::sync::Arc;
    use crate::interpreter_util::check_inited_class;
    use rust_jvm_common::classnames::ClassName;
    use crate::java::lang::class_loader::ClassLoader;
    use crate::stack_entry::StackEntry;
    use crate::JVMState;

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

        pub fn from_method_descriptor_string(jvm: &'static JVMState, frame: &StackEntry, str: crate::java::lang::string::JString, class_loader: Option<ClassLoader>) -> MethodType {
            frame.push(str.java_value());
            frame.push(class_loader.map(|x| x.java_value()).unwrap_or(JavaValue::Object(None)));
            let method_type = check_inited_class(jvm, &ClassName::method_type().into(), frame.class_pointer.loader(jvm).clone());
            crate::instructions::invoke::native::mhn_temp::run_static_or_virtual(jvm, &method_type, "fromMethodDescriptorString".to_string(), "(Ljava/lang/String;Ljava/lang/ClassLoader;)Ljava/lang/invoke/MethodType;".to_string());
            frame.pop().cast_method_type()
        }
    }
}


pub mod method_handle {
    use crate::java_values::{JavaValue, Object};
    use crate::{JVMState, StackEntry};

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
        pub fn lookup(jvm: &'static JVMState, frame: &StackEntry) -> Lookup {
            let method_handles_class = check_inited_class(jvm, &ClassName::method_handles().into(), frame.class_pointer.loader(jvm).clone());
            run_static_or_virtual(jvm, &method_handles_class, "lookup".to_string(), "()Ljava/lang/invoke/MethodHandles$Lookup;".to_string());
            frame.pop().cast_lookup()
        }
        pub fn public_lookup(jvm: &'static JVMState, frame: &StackEntry) -> Lookup {
            let method_handles_class = check_inited_class(jvm, &ClassName::method_handles().into(), frame.class_pointer.loader(jvm).clone());
            run_static_or_virtual(jvm, &method_handles_class, "publicLookup".to_string(), "()Ljava/lang/invoke/MethodHandles$Lookup;".to_string());
            frame.pop().cast_lookup()
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
        pub fn find_virtual(&self, jvm: &'static JVMState, frame: &StackEntry, obj: JClass, name: JString, mt: MethodType) -> MethodHandle {
            let lookup_class = check_inited_class(jvm, &ClassName::lookup().into(), frame.class_pointer.loader(jvm).clone());
            frame.push(self.clone().java_value());
            frame.push(obj.java_value());
            frame.push(name.java_value());
            frame.push(mt.java_value());
            run_static_or_virtual(jvm, &lookup_class, "findVirtual".to_string(), "(Ljava/lang/Class;Ljava/lang/String;Ljava/lang/invoke/MethodType;)Ljava/lang/invoke/MethodHandle;".to_string());
            frame.pop().cast_method_handle()
        }

        as_object_or_java_value!();
    }
}
