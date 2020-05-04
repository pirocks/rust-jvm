pub mod unsafe_ {
    use crate::java_values::{Object, JavaValue};
    use std::sync::Arc;
    use crate::{JVMState, StackEntry};

    use crate::interpreter_util::check_inited_class;
    use rust_jvm_common::classnames::ClassName;

    pub struct Unsafe {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_unsafe(&self) -> Unsafe {
            Unsafe { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl Unsafe {
        pub fn the_unsafe(jvm: & JVMState, frame: &StackEntry) -> Unsafe {
            let unsafe_class = check_inited_class(jvm, &ClassName::unsafe_(), frame.class_pointer.loader(jvm).clone());
            let static_vars = unsafe_class.static_vars();
            static_vars.get("theUnsafe").unwrap().clone().cast_unsafe()
        }

        as_object_or_java_value!();
    }
}