pub mod invoke;


pub mod member_name {
    use crate::java_values::{JavaValue, Object};
    use crate::java::lang::string::JString;
    use crate::instructions::invoke::native::mhn_temp::run_static_or_virtual;
    use crate::{JVMState, InterpreterStateGuard};

    use crate::interpreter_util::check_inited_class;
    use rust_jvm_common::classnames::ClassName;
    use std::sync::Arc;
    use crate::java::lang::class::JClass;
    use crate::java::lang::invoke::method_type::MethodType;

    pub struct MemberName {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_member_name(&self) -> MemberName {
            MemberName { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl MemberName {
        // private Class<?> clazz;
        // private String name;
        // private Object type;
        // private int flags;
        pub fn get_name<'l>(&self, jvm: &'static JVMState, int_state: &mut InterpreterStateGuard) -> JString {
            let member_name_class = check_inited_class(jvm, int_state, &ClassName::member_name().into(), int_state.current_loader(jvm));
            int_state.push_current_operand_stack(JavaValue::Object(self.normal_object.clone().into()));
            run_static_or_virtual(jvm, int_state, &member_name_class, "getName".to_string(), "()Ljava/lang/String;".to_string());
            int_state.pop_current_operand_stack().cast_string()
        }

        pub fn clazz(&self) -> JClass {
            self.normal_object.unwrap_normal_object().fields.borrow().get("clazz").unwrap().cast_class()
        }

        pub fn get_method_type<'l>(&self, jvm: &'static JVMState, int_state: &mut InterpreterStateGuard) -> MethodType {
            let member_name_class = check_inited_class(jvm, int_state, &ClassName::member_name().into(), int_state.current_loader(jvm));
            int_state.push_current_operand_stack(JavaValue::Object(self.normal_object.clone().into()));
            run_static_or_virtual(jvm, int_state, &member_name_class, "getMethodType".to_string(), "()Ljava/lang/invoke/MethodType;".to_string());
            int_state.pop_current_operand_stack().cast_method_type()
        }

        pub fn get_field_type<'l>(&self, jvm: &'static JVMState, int_state: &mut InterpreterStateGuard) -> JClass {
            let member_name_class = check_inited_class(jvm, int_state, &ClassName::member_name().into(), int_state.current_loader(jvm));
            int_state.push_current_operand_stack(JavaValue::Object(self.normal_object.clone().into()));
            run_static_or_virtual(jvm, int_state, &member_name_class, "getFieldType".to_string(), "()Ljava/lang/Class;".to_string());
            int_state.pop_current_operand_stack().cast_class()
        }
    }
}

pub mod class {
    use crate::java_values::{JavaValue, Object};
    use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
    use std::sync::Arc;
    use crate::java::lang::class_loader::ClassLoader;
    use crate::instructions::invoke::native::mhn_temp::run_static_or_virtual;

    use crate::{JVMState, InterpreterStateGuard};
    use rust_jvm_common::classnames::ClassName;
    use crate::class_objects::get_or_create_class_object;
    use crate::runtime_class::RuntimeClass;

    #[derive(Debug, Clone)]
    pub struct JClass {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_class(&self) -> JClass {
            assert_eq!(self.unwrap_normal_object().class_pointer.view().name(), ClassName::class());
            assert!(self.unwrap_normal_object().class_object_type.is_some());
            JClass { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl JClass {
        pub fn as_type(&self) -> PTypeView {
            self.normal_object.unwrap_normal_object().class_object_type.as_ref().unwrap().ptypeview()
        }

        pub fn as_runtime_class(&self) -> Arc<RuntimeClass> {
            self.normal_object.unwrap_normal_object().class_object_type.as_ref().unwrap().clone()
        }

        pub fn get_class_loader<'l>(&self, state: &'static JVMState, int_state: &mut InterpreterStateGuard) -> Option<ClassLoader> {
            int_state.push_current_operand_stack(JavaValue::Object(self.normal_object.clone().into()));
            run_static_or_virtual(
                state,
                int_state,
                &self.normal_object.unwrap_normal_object().class_pointer,
                "getClassLoader".to_string(),
                "()Ljava/lang/ClassLoader;".to_string(),
            );
            int_state.pop_current_operand_stack()
                .unwrap_object()
                .map(|cl| JavaValue::Object(cl.into()).cast_class_loader())
        }

        pub fn from_name<'l>(jvm: &'static JVMState, int_state: &mut InterpreterStateGuard, name: ClassName) -> JClass {
            let frame = int_state.current_frame_mut();
            let type_ = PTypeView::Ref(ReferenceTypeView::Class(name));
            let loader_arc = int_state.current_loader(jvm).clone();
            JavaValue::Object(get_or_create_class_object(jvm, &type_, int_state, loader_arc).into()).cast_class()
        }

        as_object_or_java_value!();
    }
}

pub mod class_loader {
    use std::sync::Arc;
    use crate::java_values::{Object, JavaValue};

    pub struct ClassLoader {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_class_loader(&self) -> ClassLoader {
            ClassLoader { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl ClassLoader {
        as_object_or_java_value!();
    }
}

pub mod string {
    use crate::utils::string_obj_to_string;
    use crate::java_values::Object;
    use std::sync::Arc;
    use crate::java_values::JavaValue;
    use crate::instructions::ldc::create_string_on_stack;
    use crate::{JVMState, InterpreterStateGuard};


    pub struct JString {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_string(&self) -> JString {
            JString { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl JString {
        pub fn to_rust_string(&self) -> String {
            string_obj_to_string(self.normal_object.clone().into())
        }

        pub fn from<'l>(state: &'static JVMState, int_state: &mut InterpreterStateGuard, rust_str: String) -> JString {
            create_string_on_stack(state, int_state, rust_str);
            int_state.pop_current_operand_stack().cast_string()
        }

        as_object_or_java_value!();
    }
}

pub mod integer {
    use jvmti_jni_bindings::jint;
    use crate::{JVMState, StackEntry};

    use crate::java_values::{JavaValue, Object};
    use std::sync::Arc;

    pub struct Integer {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_integer(&self) -> Integer {
            Integer { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl Integer {
        pub fn from(_state: &'static JVMState, _current_frame: &StackEntry, _i: jint) -> Integer {
            unimplemented!()
        }

        pub fn value(&self) -> jint {
            self.normal_object.unwrap_normal_object().fields.borrow().get("value").unwrap().unwrap_int()
        }

        as_object_or_java_value!();
    }
}

pub mod object {
    use crate::java_values::Object;
    use std::sync::Arc;
    use crate::java_values::JavaValue;

    pub struct JObject {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_object(&self) -> JObject {
            JObject { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl JObject {
        as_object_or_java_value!();
    }
}

pub mod thread {
    use crate::java_values::{Object, NormalObject};
    use std::sync::Arc;
    use crate::java_values::JavaValue;
    use crate::{JVMState, InterpreterStateGuard};
    use crate::stack_entry::StackEntry;
    use crate::instructions::invoke::native::mhn_temp::run_static_or_virtual;
    use crate::interpreter_util::{check_inited_class, push_new_object, run_constructor};
    use rust_jvm_common::classnames::ClassName;
    use crate::java::lang::string::JString;
    use crate::threading::{JavaThreadId, JavaThread};
    use crate::java::lang::thread_group::JThreadGroup;
    use crate::runtime_class::RuntimeClass;
    use std::cell::RefCell;
    use crate::threading::monitors::Monitor;

    #[derive(Debug, Clone)]
    pub struct JThread {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_thread(&self) -> JThread {
            JThread { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl JThread {
        pub fn invalid_thread(jvm: &'static JVMState) -> JThread{
            JThread{ normal_object: Arc::new(Object::Object(NormalObject{
                monitor: jvm.thread_state.new_monitor("invalid thread monitor".to_string()),
                fields: RefCell::new(Default::default()),
                class_pointer: Arc::new(RuntimeClass::Byte),
                class_object_type: None
            })) }
        }

        pub fn tid(&self) -> JavaThreadId {
            self.normal_object.unwrap_normal_object().fields.borrow().get("tid").unwrap().unwrap_long()
        }

        pub fn run<'l>(&self, jvm: &'static JVMState, int_state: &mut InterpreterStateGuard) {
            let thread_class = check_inited_class(jvm, int_state, &ClassName::thread().into(), int_state.current_loader(jvm).clone());
            int_state.push_current_operand_stack(JavaValue::Object(self.normal_object.clone().into()));
            run_static_or_virtual(jvm, int_state, &thread_class, "run".to_string(), "()V".to_string());
        }

        pub fn name(&self) -> JString {
            self.normal_object.lookup_field("name").cast_string()
        }

        pub fn priority(&self) -> i32 {
            self.normal_object.lookup_field("priority").unwrap_int()
        }

        pub fn set_priority(&self, priority:i32) {
            self.normal_object.unwrap_normal_object().fields.borrow_mut().insert("priority".to_string(),JavaValue::Int(priority));
        }

        pub fn daemon(&self) -> bool {
            self.normal_object.lookup_field("daemon").unwrap_int() != 0
        }


        pub fn new(jvm: &'static JVMState, int_state: &mut InterpreterStateGuard, thread_group: JThreadGroup, thread_name: String) -> JThread {
            let thread_class = check_inited_class(jvm, int_state, &ClassName::thread().into(), int_state.current_loader(jvm).clone());
            push_new_object(jvm, int_state, &thread_class, None);
            let thread_object = int_state.pop_current_operand_stack();
            let thread_name = JString::from(jvm, int_state, thread_name);
            run_constructor(jvm, int_state, thread_class, vec![thread_object.clone(), thread_group.java_value(), thread_name.java_value()],
                            "(Ljava/lang/ThreadGroup;Ljava/lang/String;)V".to_string());
            thread_object.cast_thread()
        }

        pub fn get_java_thread(&self, jvm: &'static JVMState) -> Arc<JavaThread>{
            self.try_get_java_thread(jvm).unwrap()
        }

        pub fn try_get_java_thread(&self, jvm: &'static JVMState) -> Option<Arc<JavaThread>>{
            let tid = self.tid();
            jvm.thread_state.try_get_thread_by_tid(tid)
        }

        as_object_or_java_value!();
    }
}

pub mod thread_group {
    use crate::java_values::{JavaValue, Object};
    use std::sync::Arc;
    use crate::interpreter_util::{push_new_object, check_inited_class, run_constructor};
    use rust_jvm_common::classnames::ClassName;
    use crate::{JVMState, InterpreterStateGuard};

    #[derive(Debug, Clone)]
    pub struct JThreadGroup {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_thread_group(&self) -> JThreadGroup {
            JThreadGroup { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl JThreadGroup {
        pub fn init<'l>(jvm: &'static JVMState,
                        int_state: &mut InterpreterStateGuard) -> JThreadGroup {
            let thread_group_class = check_inited_class(jvm, int_state, &ClassName::Str("java/lang/ThreadGroup".to_string()).into(), int_state.current_loader(jvm).clone());
            push_new_object(jvm, int_state, &thread_group_class, None);
            let thread_group_object = int_state.pop_current_operand_stack();
            run_constructor(jvm, int_state, thread_group_class, vec![thread_group_object.clone()],
                            "()V".to_string());
            thread_group_object.cast_thread_group()
        }

        as_object_or_java_value!();
    }
}


pub mod system;

pub mod reflect;