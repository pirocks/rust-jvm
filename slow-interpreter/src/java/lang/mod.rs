pub mod invoke;


pub mod member_name {
    use std::sync::Arc;

    use jvmti_jni_bindings::jint;
    use rust_jvm_common::classnames::ClassName;
    use type_safe_proc_macro_utils::getter_gen;

    use crate::{InterpreterStateGuard, JVMState};
    use crate::instructions::invoke::native::mhn_temp::run_static_or_virtual;
    use crate::interpreter_util::{check_inited_class, push_new_object};
    use crate::java::lang::class::JClass;
    use crate::java::lang::invoke::method_type::MethodType;
    use crate::java::lang::string::JString;
    use crate::java_values::{JavaValue, Object};

    #[derive(Clone, Debug)]
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
        pub fn get_name_func(&self, jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> JString {
            let member_name_class = check_inited_class(jvm, int_state, ClassName::member_name().into()).unwrap();
            int_state.push_current_operand_stack(JavaValue::Object(self.normal_object.clone().into()));
            run_static_or_virtual(jvm, int_state, &member_name_class, "getName".to_string(), "()Ljava/lang/String;".to_string());
            int_state.pop_current_operand_stack().cast_string()
        }

        pub fn is_static(&self, jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> bool {
            let member_name_class = check_inited_class(jvm, int_state, ClassName::member_name().into()).unwrap();
            int_state.push_current_operand_stack(JavaValue::Object(self.normal_object.clone().into()));
            run_static_or_virtual(jvm, int_state, &member_name_class, "isStatic".to_string(), "()Z".to_string());
            int_state.pop_current_operand_stack().unwrap_boolean() != 0
        }

        pub fn get_name_or_null(&self) -> Option<JString> {
            let str_jvalue = self.normal_object.unwrap_normal_object().fields_mut().get(&"name".to_string()).unwrap().clone();
            if str_jvalue.unwrap_object().is_none() {
                None
            } else {
                str_jvalue.cast_string().into()
            }
        }

        pub fn get_name(&self) -> JString {
            self.get_name_or_null().unwrap()
        }


        pub fn set_name(&self, new_val: JString) {
            self.normal_object.unwrap_normal_object().fields_mut().insert("name".to_string(), new_val.java_value());
        }

        pub fn get_clazz_or_null(&self) -> Option<JClass> {
            let possibly_null = self.normal_object.unwrap_normal_object().fields_mut().get(&"clazz".to_string()).unwrap().clone();
            if possibly_null.unwrap_object().is_none() {
                None
            } else {
                possibly_null.cast_class().into()
            }
        }

        pub fn get_clazz(&self) -> JClass {
            self.get_clazz_or_null().unwrap()
        }

        pub fn set_clazz(&self, new_val: JClass) {
            self.normal_object.unwrap_normal_object().fields_mut().insert("clazz".to_string(), new_val.java_value());
        }

        pub fn set_type(&self, new_val: MethodType) {
            self.normal_object.unwrap_normal_object().fields_mut().insert("type".to_string(), new_val.java_value());
        }


        pub fn get_type(&self) -> JavaValue {
            self.normal_object.unwrap_normal_object().fields_mut().get("type").unwrap().clone()
        }

        pub fn set_flags(&self, new_val: jint) {
            self.normal_object.unwrap_normal_object().fields_mut().insert("flags".to_string(), JavaValue::Int(new_val));
        }


        getter_gen!(flags,jint,unwrap_int);
        // pub fn get_flags(&self) -> jint {
        //     self.normal_object.unwrap_normal_object().fields_mut().get(&"flags".to_string()).unwrap().unwrap_int()
        // }

        pub fn set_resolution(&self, new_val: JavaValue) {
            self.normal_object.unwrap_normal_object().fields_mut().insert("resolution".to_string(), new_val);
        }

        pub fn get_resolution(&self) -> JavaValue {
            self.normal_object.unwrap_normal_object().fields_mut().get(&"resolution".to_string()).unwrap().clone()
        }

        pub fn clazz(&self) -> JClass {
            self.normal_object.unwrap_normal_object().fields_mut().get("clazz").unwrap().cast_class()
        }

        pub fn get_method_type(&self, jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> MethodType {
            let member_name_class = check_inited_class(jvm, int_state, ClassName::member_name().into()).unwrap();
            int_state.push_current_operand_stack(JavaValue::Object(self.normal_object.clone().into()));
            run_static_or_virtual(jvm, int_state, &member_name_class, "getMethodType".to_string(), "()Ljava/lang/invoke/MethodType;".to_string());
            int_state.pop_current_operand_stack().cast_method_type()
        }

        pub fn get_field_type(&self, jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> JClass {
            let member_name_class = check_inited_class(jvm, int_state, ClassName::member_name().into()).unwrap();
            int_state.push_current_operand_stack(JavaValue::Object(self.normal_object.clone().into()));
            run_static_or_virtual(jvm, int_state, &member_name_class, "getFieldType".to_string(), "()Ljava/lang/Class;".to_string());
            int_state.pop_current_operand_stack().cast_class()
        }

        pub fn new_member_name(jvm: &JVMState, int_state: &mut InterpreterStateGuard, clazz: JClass, name: JString, type_: MethodType, flags: jint, resolution: JavaValue) -> Self {
            let target_classfile = check_inited_class(jvm, int_state, ClassName::member_name().into()).unwrap();
            push_new_object(jvm, int_state, &target_classfile, None);
            let obj = int_state.pop_current_operand_stack().cast_member_name();
            obj.set_clazz(clazz);
            obj.set_name(name);
            obj.set_type(type_);
            obj.set_flags(flags);
            obj.set_resolution(resolution);
            obj
        }

        pub fn new_self_resolution(jvm: &JVMState, int_state: &mut InterpreterStateGuard, clazz: JClass, name: JString, type_: MethodType, flags: jint) -> Self {
            let target_classfile = check_inited_class(jvm, int_state, ClassName::member_name().into()).unwrap();
            push_new_object(jvm, int_state, &target_classfile, None);
            let obj = int_state.pop_current_operand_stack().cast_member_name();
            obj.set_clazz(clazz);
            obj.set_name(name);
            obj.set_type(type_);
            obj.set_flags(flags);
            obj.set_resolution(obj.clone().java_value());
            obj
        }

        // fn get_arc_address(&self) -> usize{
        //     let obj = self.normal_object.deref();
        //     let ptr = obj as *const Object;
        //     obj.
        // }

        as_object_or_java_value!();
    }
}

pub mod class {
    use std::sync::Arc;

    use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
    use rust_jvm_common::classnames::ClassName;

    use crate::{InterpreterStateGuard, JVMState};
    use crate::class_objects::get_or_create_class_object;
    use crate::instructions::invoke::native::mhn_temp::run_static_or_virtual;
    use crate::instructions::ldc::load_class_constant_by_type;
    use crate::java::lang::class_loader::ClassLoader;
    use crate::java_values::{JavaValue, Object};
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

        pub fn get_class_loader(&self, state: &JVMState, int_state: &mut InterpreterStateGuard) -> Option<ClassLoader> {
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

        pub fn from_name(jvm: &JVMState, int_state: &mut InterpreterStateGuard, name: ClassName) -> JClass {
            let type_ = PTypeView::Ref(ReferenceTypeView::Class(name));
            JavaValue::Object(get_or_create_class_object(jvm, &type_, int_state).unwrap().into()).cast_class()
        }

        pub fn from_type(jvm: &JVMState, int_state: &mut InterpreterStateGuard, ptype: &PTypeView) -> JClass{
            load_class_constant_by_type(jvm, int_state, &ptype);
            let res = int_state.pop_current_operand_stack().unwrap_object();
            JavaValue::Object(res).cast_class()
        }

        // pub fn from_name_suppress_class_load<'l>(jvm: &JVMState, int_state: &mut InterpreterStateGuard, name: ClassName) -> JClass {
        //     let type_ = PTypeView::Ref(ReferenceTypeView::Class(name));
        //     let loader_arc = int_state.current_loader(jvm).clone();
        //     JavaValue::Object(get_or_create_class_object(jvm, &type_, int_state, loader_arc).into()).cast_class()
        // }

        as_object_or_java_value!();
    }
}

pub mod class_loader {
    use std::sync::Arc;

    use by_address::ByAddress;

    use classfile_view::loading::LoaderName;
    use rust_jvm_common::classnames::ClassName;

    use crate::instructions::invoke::native::mhn_temp::run_static_or_virtual;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::interpreter_util::{check_inited_class, check_inited_class_override_loader};
    use crate::java::lang::class::JClass;
    use crate::java::lang::string::JString;
    use crate::java_values::{JavaValue, Object};
    use crate::jvm_state::JVMState;

    #[derive(Clone)]
    pub struct ClassLoader {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_class_loader(&self) -> ClassLoader {
            ClassLoader { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl ClassLoader {
        pub fn to_jvm_loader(&self, jvm: &JVMState) -> LoaderName {
            let mut loaders_guard = jvm.class_loaders.write().unwrap();
            let loader_index_lookup = loaders_guard.get_by_right(&ByAddress(self.normal_object.clone()));
            LoaderName::UserDefinedLoader(match loader_index_lookup {
                Some(x) => *x,
                None => {
                    let new_loader_id = loaders_guard.len();
                    loaders_guard.insert(new_loader_id, ByAddress(self.normal_object.clone()));
                    //todo this whole mess needs a register class loader function which addes to approprate classes data structure
                    new_loader_id
                },
            })
        }

        pub fn load_class(&self, jvm: &JVMState, int_state: &mut InterpreterStateGuard, name: JString) -> JClass {
            int_state.push_current_operand_stack(self.clone().java_value());
            int_state.push_current_operand_stack(name.java_value());
            let class_loader = check_inited_class_override_loader(jvm, int_state, &ClassName::classloader().into(), LoaderName::BootstrapLoader).unwrap();
            run_static_or_virtual(
                jvm,
                int_state,
                &class_loader,
                "loadClass".to_string(),
                "(Ljava/lang/String;)Ljava/lang/Class;".to_string(),
            );
            assert!(int_state.throw().is_none());
            int_state.pop_current_operand_stack().cast_class()
        }

        as_object_or_java_value!();
    }
}

pub mod string {
    use std::cell::UnsafeCell;
    use std::sync::Arc;

    use classfile_view::loading::LoaderName;
    use rust_jvm_common::classfile::ConstantKind::LiveObject;
    use rust_jvm_common::classnames::ClassName;

    use crate::{InterpreterStateGuard, JVMState};
    use crate::instructions::invoke::native::mhn_temp::run_static_or_virtual;
    use crate::interpreter_util::{check_inited_class, check_inited_class_override_loader, push_new_object, run_constructor};
    use crate::java_values::{ArrayObject, JavaValue};
    use crate::java_values::Object;
    use crate::sun::misc::unsafe_::Unsafe;
    use crate::utils::string_obj_to_string;

    #[derive(Clone)]
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

        pub fn from_rust(jvm: &JVMState, int_state: &mut InterpreterStateGuard, rust_str: String) -> JString {
            let string_class = check_inited_class_override_loader(jvm, int_state, &ClassName::string().into(), LoaderName::BootstrapLoader).unwrap();
            push_new_object(jvm, int_state, &string_class, None);
            // dbg!(int_state.current_frame().local_vars());
            // dbg!(int_state.current_frame().operand_stack());
            let string_object = int_state.pop_current_operand_stack();

            let vec1 = rust_str.chars().map(|c| JavaValue::Char(c as u16)).collect::<Vec<JavaValue>>();
            let array_object = ArrayObject {
                elems: UnsafeCell::new(vec1),
                elem_type: ClassName::string().into(),
                monitor: jvm.thread_state.new_monitor("monitor for a string".to_string()),
            };
            //todo what about check_initied_class for this array type
            let array = JavaValue::Object(Some(Arc::new(Object::Array(array_object))));
            run_constructor(jvm, int_state, string_class, vec![string_object.clone(), array],
                            "([C)V".to_string());
            string_object.cast_string()
        }

        pub fn intern(&self, jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> JString {
            int_state.push_current_operand_stack(self.clone().java_value());
            let thread_class = check_inited_class(jvm, int_state, ClassName::string().into()).unwrap();
            run_static_or_virtual(
                jvm,
                int_state,
                &thread_class,
                "intern".to_string(),
                "()Ljava/lang/String;".to_string(),
            );
            int_state.pop_current_operand_stack().cast_string()
        }

        as_object_or_java_value!();
    }
}

pub mod integer {
    use std::sync::Arc;

    use jvmti_jni_bindings::jint;

    use crate::{JVMState, StackEntry};
    use crate::java_values::{JavaValue, Object};

    pub struct Integer {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_integer(&self) -> Integer {
            Integer { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl Integer {
        pub fn from(_state: &JVMState, _current_frame: &StackEntry, _i: jint) -> Integer {
            unimplemented!()
        }

        pub fn value(&self) -> jint {
            self.normal_object.unwrap_normal_object().fields_mut().get("value").unwrap().unwrap_int()
        }

        as_object_or_java_value!();
    }
}

pub mod object {
    use std::sync::Arc;

    use crate::java_values::JavaValue;
    use crate::java_values::Object;

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
    use std::cell::UnsafeCell;
    use std::sync::Arc;

    use jvmti_jni_bindings::{jboolean, jint};
    use rust_jvm_common::classnames::ClassName;

    use crate::{InterpreterStateGuard, JVMState};
    use crate::instructions::invoke::native::mhn_temp::run_static_or_virtual;
    use crate::interpreter_util::{check_inited_class, push_new_object, run_constructor};
    use crate::java::lang::class_loader::ClassLoader;
    use crate::java::lang::string::JString;
    use crate::java::lang::thread_group::JThreadGroup;
    use crate::java_values::{NormalObject, Object};
    use crate::java_values::JavaValue;
    use crate::runtime_class::RuntimeClass;
    use crate::threading::{JavaThread, JavaThreadId};

    #[derive(Debug, Clone)]
    pub struct JThread {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        //todo these shouldn't silently error
        pub fn cast_thread(&self) -> JThread {
            JThread { normal_object: self.unwrap_object_nonnull() }
        }

        pub fn try_cast_thread(&self) -> Option<JThread> {
            match self.try_unwrap_normal_object() {
                Some(_normal_object) => {
                    // if normal_object.class_pointer.view().name() == ClassName::thread() { //todo add this kind of check back at some point
                    JThread { normal_object: self.unwrap_object_nonnull() }.into()
                    // }
                    // None
                }
                None => None
            }
        }
    }

    impl JThread {
        pub fn invalid_thread(jvm: &JVMState) -> JThread {
            JThread {
                normal_object: Arc::new(Object::Object(NormalObject {
                    monitor: jvm.thread_state.new_monitor("invalid thread monitor".to_string()),
                    fields: UnsafeCell::new(Default::default()),
                    class_pointer: Arc::new(RuntimeClass::Byte),
                    class_object_type: None,
                }))
            }
        }

        pub fn tid(&self) -> JavaThreadId {
            match self.normal_object.unwrap_normal_object().fields_mut().get("tid") {
                Some(x) => x,
                None => {
                    dbg!(&self.normal_object);
                    panic!()
                }
            }.unwrap_long()
        }

        pub fn run(&self, jvm: &JVMState, int_state: &mut InterpreterStateGuard) {
            let thread_class = self.normal_object.unwrap_normal_object().class_pointer.clone();
            int_state.push_current_operand_stack(JavaValue::Object(self.normal_object.clone().into()));
            run_static_or_virtual(jvm, int_state, &thread_class, "run".to_string(), "()V".to_string());
        }

        pub fn name(&self) -> JString {
            self.normal_object.lookup_field("name").cast_string()
        }

        pub fn priority(&self) -> i32 {
            self.normal_object.lookup_field("priority").unwrap_int()
        }

        pub fn set_priority(&self, priority: i32) {
            self.normal_object.unwrap_normal_object().fields_mut().insert("priority".to_string(), JavaValue::Int(priority));
        }

        pub fn daemon(&self) -> bool {
            self.normal_object.lookup_field("daemon").unwrap_int() != 0
        }

        pub fn set_thread_status(&self, thread_status: jint) {
            self.normal_object.unwrap_normal_object().fields_mut().insert("threadStatus".to_string(), JavaValue::Int(thread_status));
        }


        pub fn new(jvm: &JVMState, int_state: &mut InterpreterStateGuard, thread_group: JThreadGroup, thread_name: String) -> JThread {
            let thread_class = check_inited_class(jvm, int_state, ClassName::thread().into()).unwrap();
            push_new_object(jvm, int_state, &thread_class, None);
            let thread_object = int_state.pop_current_operand_stack();
            let thread_name = JString::from_rust(jvm, int_state, thread_name);
            run_constructor(jvm, int_state, thread_class, vec![thread_object.clone(), thread_group.java_value(), thread_name.java_value()],
                            "(Ljava/lang/ThreadGroup;Ljava/lang/String;)V".to_string());
            thread_object.cast_thread()
        }

        pub fn get_java_thread(&self, jvm: &JVMState) -> Arc<JavaThread> {
            self.try_get_java_thread(jvm).unwrap()
        }

        pub fn try_get_java_thread(&self, jvm: &JVMState) -> Option<Arc<JavaThread>> {
            let tid = self.tid();
            jvm.thread_state.try_get_thread_by_tid(tid)
        }

        pub fn is_alive(&self, jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> jboolean {
            // assert_eq!(self.normal_object.unwrap_normal_object().class_pointer.view().name(), ClassName::thread());
            let thread_class = check_inited_class(jvm, int_state, ClassName::thread().into()).unwrap();
            int_state.push_current_operand_stack(self.clone().java_value());
            // dbg!(&self.normal_object);
            run_static_or_virtual(
                jvm,
                int_state,
                &thread_class,
                "isAlive".to_string(),
                "()Z".to_string(),
            );
            int_state.pop_current_operand_stack()
                .unwrap_boolean()
        }


        pub fn current_thread(jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> JThread {
            let thread_class = check_inited_class(jvm, int_state, ClassName::thread().into()).unwrap();
            run_static_or_virtual(
                jvm,
                int_state,
                &thread_class,
                "currentThread".to_string(),
                "()Ljava/lang/Thread;".to_string(),
            );
            int_state.pop_current_operand_stack().cast_thread()
        }


        pub fn get_context_class_loader(&self, jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> Option<ClassLoader> {
            let thread_class = check_inited_class(jvm, int_state, ClassName::thread().into()).unwrap();
            int_state.push_current_operand_stack(self.clone().java_value());
            run_static_or_virtual(
                jvm,
                int_state,
                &thread_class,
                "getContextClassLoader".to_string(),
                "()Ljava/lang/ClassLoader;".to_string(),
            );
            let res = int_state.pop_current_operand_stack();
            if res.unwrap_object().is_none() {
                return None
            }
            res.cast_class_loader().into()
        }

        as_object_or_java_value!();
    }
}

pub mod thread_group {
    use std::sync::Arc;

    use jvmti_jni_bindings::{jboolean, jint};
    use rust_jvm_common::classnames::ClassName;

    use crate::{InterpreterStateGuard, JVMState};
    use crate::interpreter_util::{check_inited_class, push_new_object, run_constructor};
    use crate::java::lang::string::JString;
    use crate::java::lang::thread::JThread;
    use crate::java_values::{JavaValue, Object};
    use crate::runtime_class::RuntimeClass;

    #[derive(Debug, Clone)]
    pub struct JThreadGroup {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_thread_group(&self) -> JThreadGroup {
            JThreadGroup { normal_object: self.unwrap_object_nonnull() }
        }

        pub fn try_cast_thread_group(&self) -> Option<JThreadGroup> {
            match self.try_unwrap_normal_object() {
                Some(normal_object) => {
                    if normal_object.class_pointer.view().name() == ClassName::thread_group() {
                        return JThreadGroup { normal_object: self.unwrap_object_nonnull() }.into();
                    }
                    None
                }
                None => None
            }
        }
    }

    impl JThreadGroup {
        pub fn init(jvm: &JVMState,
                    int_state: &mut InterpreterStateGuard, thread_group_class: Arc<RuntimeClass>) -> JThreadGroup {
            push_new_object(jvm, int_state, &thread_group_class, None);
            let thread_group_object = int_state.pop_current_operand_stack();
            run_constructor(jvm, int_state, thread_group_class, vec![thread_group_object.clone()],
                            "()V".to_string());
            thread_group_object.cast_thread_group()
        }

        pub fn threads(&self) -> Vec<Option<JThread>> {
            unsafe {
                self.normal_object.lookup_field("threads").unwrap_array().elems.get().as_ref().unwrap().iter().map(|thread|
                    {
                        match thread.unwrap_object() {
                            None => None,
                            Some(t) => JavaValue::Object(t.into()).cast_thread().into(),
                        }
                    }
                ).collect()
            }
        }

        pub fn threads_non_null(&self) -> Vec<JThread> {
            self.threads().into_iter().flatten().collect()
        }

        pub fn name(&self) -> JString {
            self.normal_object.lookup_field("name").cast_string()
        }

        pub fn daemon(&self) -> jboolean {
            self.normal_object.lookup_field("daemon").unwrap_boolean()
        }

        pub fn max_priority(&self) -> jint {
            self.normal_object.lookup_field("maxPriority").unwrap_int()
        }

        pub fn parent(&self) -> Option<JThreadGroup> {
            self.normal_object.lookup_field("parent").try_cast_thread_group()
        }

        as_object_or_java_value!();
    }
}


pub mod class_not_found_exception {
    use std::sync::Arc;

    use rust_jvm_common::classnames::ClassName;

    use crate::interpreter_state::InterpreterStateGuard;
    use crate::interpreter_util::{check_inited_class, push_new_object, run_constructor};
    use crate::java::lang::string::JString;
    use crate::java_values::JavaValue;
    use crate::java_values::Object;
    use crate::jvm_state::JVMState;

    pub struct ClassNotFoundException {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_class_not_found_exception(&self) -> ClassNotFoundException {
            ClassNotFoundException { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl ClassNotFoundException {
        as_object_or_java_value!();

        pub fn new(jvm: &JVMState, int_state: &mut InterpreterStateGuard, class: JString) -> ClassNotFoundException {
            let class_not_found_class = check_inited_class(jvm, int_state, ClassName::Str("java/lang/ClassNotFoundException".to_string()).into()).unwrap();
            push_new_object(jvm, int_state, &class_not_found_class, None);
            let this = int_state.pop_current_operand_stack();
            run_constructor(jvm, int_state, class_not_found_class, vec![this.clone(), class.java_value()],
                            "(Ljava/lang/String;)V".to_string());
            this.cast_class_not_found_exception()
        }
    }
}

pub mod system;

pub mod reflect;