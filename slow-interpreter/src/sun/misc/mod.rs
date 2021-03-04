pub mod unsafe_ {
    use std::sync::Arc;

    use rust_jvm_common::classnames::ClassName;

    use crate::{InterpreterStateGuard, JVMState};
    use crate::class_loading::assert_inited_or_initing_class;
    use crate::instructions::invoke::native::mhn_temp::run_static_or_virtual;
    use crate::java::lang::reflect::field::Field;
    use crate::java_values::{JavaValue, Object};

    pub struct Unsafe {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_unsafe(&self) -> Unsafe {
            Unsafe { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl Unsafe {
        pub fn the_unsafe(jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> Unsafe {
            let unsafe_class = assert_inited_or_initing_class(jvm, int_state, ClassName::unsafe_().into());
            let static_vars = unsafe_class.static_vars();
            static_vars.get("theUnsafe").unwrap().clone().cast_unsafe()
        }

        pub fn object_field_offset(&self, jvm: &JVMState, int_state: &mut InterpreterStateGuard, field: Field) -> JavaValue {
            let desc_str = "(Ljava/lang/reflect/Field;)J";
            int_state.push_current_operand_stack(JavaValue::Object(self.normal_object.clone().into()));
            int_state.push_current_operand_stack(field.java_value());
            let rc = self.normal_object.unwrap_normal_object().class_pointer.clone();
            run_static_or_virtual(jvm, int_state, &rc, "objectFieldOffset".to_string(), desc_str.to_string());
            int_state.pop_current_operand_stack()
        }

        as_object_or_java_value!();
    }
}

pub mod launcher {
    use std::sync::Arc;

    use rust_jvm_common::classnames::ClassName;

    use crate::class_loading::check_initing_or_inited_class;
    use crate::instructions::invoke::native::mhn_temp::run_static_or_virtual;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::java::lang::class_loader::ClassLoader;
    use crate::java_values::{JavaValue, Object};
    use crate::jvm_state::JVMState;

    pub struct Launcher {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_launcher(&self) -> Launcher {
            Launcher { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl Launcher {
        pub fn get_launcher(jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> Launcher {
            let launcher = check_initing_or_inited_class(jvm, int_state, ClassName::Str("sun/misc/Launcher".to_string()).into()).unwrap();//todo
            run_static_or_virtual(jvm, int_state, &launcher, "getLauncher".to_string(), "()Lsun/misc/Launcher;".to_string());
            int_state.pop_current_operand_stack().cast_launcher()
        }

        pub fn get_loader(&self, jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> ClassLoader {
            let launcher = check_initing_or_inited_class(jvm, int_state, ClassName::Str("sun/misc/Launcher".to_string()).into()).unwrap();//todo
            int_state.push_current_operand_stack(JavaValue::Object(self.normal_object.clone().into()));
            run_static_or_virtual(jvm, int_state, &launcher, "getClassLoader".to_string(), "()Ljava/lang/ClassLoader;".to_string());
            int_state.pop_current_operand_stack().cast_class_loader()
        }

        as_object_or_java_value!();
    }

    pub mod ext_class_loader {
        use std::sync::Arc;

        use rust_jvm_common::classnames::ClassName;

        use crate::class_loading::check_initing_or_inited_class;
        use crate::instructions::invoke::native::mhn_temp::run_static_or_virtual;
        use crate::interpreter_state::InterpreterStateGuard;
        use crate::java_values::{JavaValue, Object};
        use crate::jvm_state::JVMState;

        pub struct ExtClassLoader {
            normal_object: Arc<Object>
        }

        impl JavaValue {
            pub fn cast_ext_class_launcher(&self) -> ExtClassLoader {
                ExtClassLoader { normal_object: self.unwrap_object_nonnull() }
            }
        }

        impl ExtClassLoader {
            pub fn get_ext_class_loader(jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> ExtClassLoader {
                let ext_class_loader = check_initing_or_inited_class(jvm, int_state, ClassName::new("sun/misc/Launcher$ExtClassLoader").into()).unwrap();//todo
                run_static_or_virtual(jvm, int_state, &ext_class_loader, "getExtClassLoader".to_string(), "()Lsun/misc/Launcher;".to_string());
                int_state.pop_current_operand_stack().cast_ext_class_launcher()
            }

            as_object_or_java_value!();
        }
    }
}