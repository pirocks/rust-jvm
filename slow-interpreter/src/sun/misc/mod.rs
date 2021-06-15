pub mod unsafe_ {
    use std::sync::Arc;

    use rust_jvm_common::classnames::ClassName;

    use crate::{InterpreterStateGuard, JVMState};
    use crate::class_loading::assert_inited_or_initing_class;
    use crate::interpreter::WasException;
    use crate::java::lang::reflect::field::Field;
    use crate::java_values::{JavaValue, Object};
    use crate::utils::run_static_or_virtual;

    pub struct Unsafe<'gc_life> {
        normal_object: Arc<Object<'gc_life>>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_unsafe(&self) -> Unsafe {
            Unsafe { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> Unsafe<'gc_life> {
        pub fn the_unsafe<'l, 'k : 'l>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>) -> Unsafe<'gc_life> {
            let unsafe_class = assert_inited_or_initing_class(jvm, ClassName::unsafe_().into());
            let static_vars = unsafe_class.static_vars();
            static_vars.get("theUnsafe").unwrap().clone().cast_unsafe()
        }

        pub fn object_field_offset<'l, 'k : 'l>(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>, field: Field<'gc_life>) -> Result<JavaValue<'gc_life>, WasException> {
            let desc_str = "(Ljava/lang/reflect/Field;)J";
            int_state.push_current_operand_stack(JavaValue::Object(todo!()/*self.normal_object.clone().into()*/));
            int_state.push_current_operand_stack(field.java_value());
            let rc = self.normal_object.unwrap_normal_object().objinfo.class_pointer.clone();
            run_static_or_virtual(jvm, int_state, &rc, "objectFieldOffset".to_string(), desc_str.to_string())?;
            Ok(int_state.pop_current_operand_stack(ClassName::object().into()))
        }

        as_object_or_java_value!();
    }
}

pub mod launcher {
    use std::sync::Arc;

    use rust_jvm_common::classnames::ClassName;

    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::java::lang::class_loader::ClassLoader;
    use crate::java_values::{JavaValue, Object};
    use crate::jvm_state::JVMState;
    use crate::utils::run_static_or_virtual;

    pub struct Launcher<'gc_life> {
        normal_object: Arc<Object<'gc_life>>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_launcher(&self) -> Launcher {
            Launcher { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> Launcher<'gc_life> {
        pub fn get_launcher<'l, 'k : 'l>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>) -> Result<Launcher<'gc_life>, WasException> {
            let launcher = check_initing_or_inited_class(jvm, int_state, ClassName::Str("sun/misc/Launcher".to_string()).into())?;
            run_static_or_virtual(jvm, int_state, &launcher, "getLauncher".to_string(), "()Lsun/misc/Launcher;".to_string())?;
            Ok(int_state.pop_current_operand_stack(ClassName::object().into()).cast_launcher())
        }

        pub fn get_loader<'l, 'k : 'l>(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>) -> Result<ClassLoader<'gc_life>, WasException> {
            let launcher = check_initing_or_inited_class(jvm, int_state, ClassName::Str("sun/misc/Launcher".to_string()).into())?;
            int_state.push_current_operand_stack(JavaValue::Object(todo!()/*self.normal_object.clone().into()*/));
            run_static_or_virtual(jvm, int_state, &launcher, "getClassLoader".to_string(), "()Ljava/lang/ClassLoader;".to_string())?;
            Ok(int_state.pop_current_operand_stack(ClassName::classloader().into()).cast_class_loader())
        }

        as_object_or_java_value!();
    }

    pub mod ext_class_loader {
        use std::sync::Arc;

        use rust_jvm_common::classnames::ClassName;

        use crate::class_loading::check_initing_or_inited_class;
        use crate::interpreter::WasException;
        use crate::interpreter_state::InterpreterStateGuard;
        use crate::java_values::{JavaValue, Object};
        use crate::jvm_state::JVMState;
        use crate::utils::run_static_or_virtual;

        pub struct ExtClassLoader<'gc_life> {
            normal_object: Arc<Object<'gc_life>>,
        }

        impl<'gc_life> JavaValue<'gc_life> {
            pub fn cast_ext_class_launcher(&self) -> ExtClassLoader {
                ExtClassLoader { normal_object: self.unwrap_object_nonnull() }
            }
        }

        impl<'gc_life> ExtClassLoader<'gc_life> {
            pub fn get_ext_class_loader<'l, 'k : 'l>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>) -> Result<ExtClassLoader<'gc_life>, WasException> {
                let ext_class_loader = check_initing_or_inited_class(jvm, int_state, ClassName::new("sun/misc/Launcher$ExtClassLoader").into())?;
                run_static_or_virtual(jvm, int_state, &ext_class_loader, "getExtClassLoader".to_string(), "()Lsun/misc/Launcher;".to_string())?;
                Ok(int_state.pop_current_operand_stack(ClassName::classloader().into()).cast_ext_class_launcher())
            }

            as_object_or_java_value!();
        }
    }
}