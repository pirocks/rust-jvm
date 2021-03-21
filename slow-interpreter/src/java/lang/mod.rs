pub mod invoke;


pub mod throwable {
    use std::sync::Arc;

    use rust_jvm_common::classnames::ClassName;

    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::java_values::{JavaValue, Object};
    use crate::jvm_state::JVMState;
    use crate::utils::run_static_or_virtual;

    #[derive(Clone, Debug)]
    pub struct Throwable {
        pub(crate) normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_throwable(&self) -> Throwable {
            Throwable { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl Throwable {
        pub fn print_stack_trace(&self, jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> Result<(), WasException> {
            let throwable_class = check_initing_or_inited_class(jvm, int_state, ClassName::throwable().into()).expect("Throwable isn't inited?");
            int_state.push_current_operand_stack(JavaValue::Object(self.normal_object.clone().into()));
            run_static_or_virtual(jvm, int_state, &throwable_class, "printStackTrace".to_string(), "()V".to_string())?;
            Ok(())
        }
        as_object_or_java_value!();
    }
}


pub mod stack_trace_element {
    use std::sync::Arc;

    use jvmti_jni_bindings::jint;
    use rust_jvm_common::classnames::ClassName;

    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::interpreter_util::{push_new_object, run_constructor};
    use crate::java::lang::string::JString;
    use crate::java_values::{JavaValue, Object};
    use crate::jvm_state::JVMState;

    #[derive(Clone, Debug)]
    pub struct StackTraceElement {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_stack_trace_element(&self) -> StackTraceElement {
            StackTraceElement { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl StackTraceElement {
        pub fn new(jvm: &JVMState, int_state: &mut InterpreterStateGuard, declaring_class: JString, method_name: JString, file_name: JString, line_number: jint) -> Result<StackTraceElement, WasException> {
            let class_ = check_initing_or_inited_class(jvm, int_state, ClassName::new("java/lang/StackTraceElement").into())?;
            push_new_object(jvm, int_state, &class_);
            let res = int_state.pop_current_operand_stack();
            let full_args = vec![res.clone(), declaring_class.java_value(), method_name.java_value(), file_name.java_value(), JavaValue::Int(
                line_number
            )];
            run_constructor(jvm, int_state, class_, full_args, "(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;I)V".to_string())?;
            Ok(res.cast_stack_trace_element())
        }

        as_object_or_java_value!();
    }
}

pub mod member_name {
    use std::sync::Arc;

    use jvmti_jni_bindings::jint;
    use rust_jvm_common::classnames::ClassName;
    use type_safe_proc_macro_utils::getter_gen;

    use crate::{InterpreterStateGuard, JVMState};
    use crate::class_loading::{assert_inited_or_initing_class, check_initing_or_inited_class};
    use crate::interpreter::WasException;
    use crate::interpreter_util::{push_new_object, run_constructor};
    use crate::java::lang::class::JClass;
    use crate::java::lang::invoke::method_type::MethodType;
    use crate::java::lang::reflect::field::Field;
    use crate::java::lang::reflect::method::Method;
    use crate::java::lang::string::JString;
    use crate::java_values::{JavaValue, Object};
    use crate::utils::run_static_or_virtual;

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
        pub fn get_name_func(&self, jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> Result<Option<JString>, WasException> {
            let member_name_class = assert_inited_or_initing_class(jvm, int_state, ClassName::member_name().into());
            int_state.push_current_operand_stack(JavaValue::Object(self.normal_object.clone().into()));
            run_static_or_virtual(jvm, int_state, &member_name_class, "getName".to_string(), "()Ljava/lang/String;".to_string())?;
            Ok(int_state.pop_current_operand_stack().cast_string())
        }

        pub fn is_static(&self, jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> Result<bool, WasException> {
            let member_name_class = assert_inited_or_initing_class(jvm, int_state, ClassName::member_name().into());
            int_state.push_current_operand_stack(JavaValue::Object(self.normal_object.clone().into()));
            run_static_or_virtual(jvm, int_state, &member_name_class, "isStatic".to_string(), "()Z".to_string())?;
            Ok(int_state.pop_current_operand_stack().unwrap_boolean() != 0)
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

        pub fn set_resolution(&self, new_val: JavaValue) {
            self.normal_object.unwrap_normal_object().fields_mut().insert("resolution".to_string(), new_val);
        }

        pub fn get_resolution(&self) -> JavaValue {
            self.normal_object.unwrap_normal_object().fields_mut().get(&"resolution".to_string()).unwrap().clone()
        }

        pub fn clazz(&self) -> Option<JClass> {
            self.normal_object.unwrap_normal_object().fields_mut().get("clazz").unwrap().cast_class()
        }

        pub fn get_method_type(&self, jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> Result<MethodType, WasException> {
            let member_name_class = assert_inited_or_initing_class(jvm, int_state, ClassName::member_name().into());
            int_state.push_current_operand_stack(JavaValue::Object(self.normal_object.clone().into()));
            run_static_or_virtual(jvm, int_state, &member_name_class, "getMethodType".to_string(), "()Ljava/lang/invoke/MethodType;".to_string())?;
            Ok(int_state.pop_current_operand_stack().cast_method_type())
        }

        pub fn get_field_type(&self, jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> Result<Option<JClass>, WasException> {
            let member_name_class = assert_inited_or_initing_class(jvm, int_state, ClassName::member_name().into());
            int_state.push_current_operand_stack(JavaValue::Object(self.normal_object.clone().into()));
            run_static_or_virtual(jvm, int_state, &member_name_class, "getFieldType".to_string(), "()Ljava/lang/Class;".to_string())?;
            Ok(int_state.pop_current_operand_stack().cast_class())
        }

        pub fn new_from_field(jvm: &JVMState, int_state: &mut InterpreterStateGuard, field: Field) -> Result<Self, WasException> {
            let member_class = check_initing_or_inited_class(jvm, int_state, ClassName::member_name().into())?;
            push_new_object(jvm, int_state, &member_class);
            let res = int_state.pop_current_operand_stack();
            run_constructor(jvm, int_state, member_class, vec![res.clone(), field.java_value()], "(Ljava/lang/reflect/Field;)V".to_string())?;
            Ok(res.cast_member_name())
        }

        pub fn new_from_method(jvm: &JVMState, int_state: &mut InterpreterStateGuard, method: Method) -> Result<Self, WasException> {
            let member_class = check_initing_or_inited_class(jvm, int_state, ClassName::member_name().into())?;
            push_new_object(jvm, int_state, &member_class);
            let res = int_state.pop_current_operand_stack();
            run_constructor(jvm, int_state, member_class, vec![res.clone(), method.java_value()], "(Ljava/lang/reflect/Method;)V".to_string())?;
            Ok(res.cast_member_name())
        }

        as_object_or_java_value!();
    }
}

pub mod class {
    use std::sync::Arc;

    use by_address::ByAddress;

    use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
    use rust_jvm_common::classnames::ClassName;

    use crate::{InterpreterStateGuard, JVMState};
    use crate::class_loading::check_initing_or_inited_class;
    use crate::class_objects::get_or_create_class_object;
    use crate::instructions::ldc::load_class_constant_by_type;
    use crate::interpreter::WasException;
    use crate::interpreter_util::{push_new_object, run_constructor};
    use crate::java::lang::class_loader::ClassLoader;
    use crate::java::lang::string::JString;
    use crate::java_values::{JavaValue, Object};
    use crate::runtime_class::RuntimeClass;
    use crate::utils::run_static_or_virtual;

    #[derive(Debug, Clone)]
    pub struct JClass {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_class(&self) -> Option<JClass> {
            Some(JClass { normal_object: self.unwrap_object()? })
        }
    }

    impl JClass {
        pub fn as_type(&self, jvm: &JVMState) -> PTypeView {
            self.as_runtime_class(jvm).ptypeview()
        }

        pub fn as_runtime_class(&self, jvm: &JVMState) -> Arc<RuntimeClass> {
            jvm.classes.read().unwrap().class_object_pool.get_by_left(&ByAddress(self.normal_object.clone())).unwrap().clone().0
        }

        pub fn get_class_loader(&self, state: &JVMState, int_state: &mut InterpreterStateGuard) -> Result<Option<ClassLoader>, WasException> {
            int_state.push_current_operand_stack(JavaValue::Object(self.normal_object.clone().into()));
            run_static_or_virtual(
                state,
                int_state,
                &self.normal_object.unwrap_normal_object().class_pointer,
                "getClassLoader".to_string(),
                "()Ljava/lang/ClassLoader;".to_string(),
            )?;
            Ok(int_state.pop_current_operand_stack()
                .unwrap_object()
                .map(|cl| JavaValue::Object(cl.into()).cast_class_loader()))
        }

        pub fn new_bootstrap_loader(jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> Result<Self, WasException> {
            let class_class = check_initing_or_inited_class(jvm, int_state, ClassName::class().into())?;
            push_new_object(jvm, int_state, &class_class);
            let res = int_state.pop_current_operand_stack();
            run_constructor(jvm, int_state, class_class, vec![res.clone(), JavaValue::Object(None)], "(Ljava/lang/ClassLoader;)V".to_string())?;
            Ok(res.cast_class().unwrap())
        }


        pub fn new(jvm: &JVMState, int_state: &mut InterpreterStateGuard, loader: ClassLoader) -> Result<Self, WasException> {
            let class_class = check_initing_or_inited_class(jvm, int_state, ClassName::class().into())?;
            push_new_object(jvm, int_state, &class_class);
            let res = int_state.pop_current_operand_stack();
            run_constructor(jvm, int_state, class_class, vec![res.clone(), loader.java_value()], "(Ljava/lang/ClassLoader;)V".to_string())?;
            Ok(res.cast_class().unwrap())
        }

        pub fn from_name(jvm: &JVMState, int_state: &mut InterpreterStateGuard, name: ClassName) -> JClass {
            let type_ = PTypeView::Ref(ReferenceTypeView::Class(name));
            JavaValue::Object(get_or_create_class_object(jvm, type_, int_state).unwrap().into()).cast_class().unwrap()
        }

        pub fn from_type(jvm: &JVMState, int_state: &mut InterpreterStateGuard, ptype: PTypeView) -> Result<JClass, WasException> {
            load_class_constant_by_type(jvm, int_state, ptype)?;
            let res = int_state.pop_current_operand_stack().unwrap_object();
            Ok(JavaValue::Object(res).cast_class().unwrap())
        }

        pub fn get_name(&self, jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> Result<JString, WasException> {
            int_state.push_current_operand_stack(self.clone().java_value());
            let class_class = check_initing_or_inited_class(jvm, int_state, ClassName::class().into()).unwrap();
            run_static_or_virtual(
                jvm,
                int_state,
                &class_class,
                "getName".to_string(),
                "()Ljava/lang/String;".to_string(),
            )?;
            Ok(int_state.pop_current_operand_stack().cast_string().expect("classes are known to have non-null names"))
        }

        pub fn set_name_(&self, name: JString) {
            let normal_object = self.normal_object.unwrap_normal_object();
            normal_object.fields_mut().insert("name".to_string(), name.java_value());
        }

        as_object_or_java_value!();
    }
}

pub mod class_loader {
    use std::sync::Arc;

    use by_address::ByAddress;

    use classfile_view::loading::LoaderName;
    use rust_jvm_common::classnames::ClassName;

    use crate::class_loading::assert_inited_or_initing_class;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::java::lang::class::JClass;
    use crate::java::lang::string::JString;
    use crate::java_values::{JavaValue, Object};
    use crate::jvm_state::JVMState;
    use crate::utils::run_static_or_virtual;

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
                    assert!(!loaders_guard.contains_left(&new_loader_id));
                    loaders_guard.insert(new_loader_id, ByAddress(self.normal_object.clone()));
                    //todo this whole mess needs a register class loader function which addes to approprate classes data structure
                    new_loader_id
                }
            })
        }

        pub fn load_class(&self, jvm: &JVMState, int_state: &mut InterpreterStateGuard, name: JString) -> Result<JClass, WasException> {
            int_state.push_current_operand_stack(self.clone().java_value());
            int_state.push_current_operand_stack(name.java_value());
            let class_loader = assert_inited_or_initing_class(jvm, int_state, ClassName::classloader().into());
            run_static_or_virtual(
                jvm,
                int_state,
                &class_loader,
                "loadClass".to_string(),
                "(Ljava/lang/String;)Ljava/lang/Class;".to_string(),
            )?;
            assert!(int_state.throw().is_none());
            Ok(int_state.pop_current_operand_stack().cast_class().unwrap())
        }

        as_object_or_java_value!();
    }
}

pub mod string {
    use std::cell::UnsafeCell;
    use std::sync::Arc;

    use jvmti_jni_bindings::{jchar, jint};
    use rust_jvm_common::classnames::ClassName;

    use crate::{InterpreterStateGuard, JVMState};
    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter::WasException;
    use crate::interpreter_util::{push_new_object, run_constructor};
    use crate::java_values::{ArrayObject, JavaValue};
    use crate::java_values::Object;
    use crate::utils::run_static_or_virtual;
    use crate::utils::string_obj_to_string;

    #[derive(Clone)]
    pub struct JString {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_string(&self) -> Option<JString> {
            Some(JString { normal_object: self.unwrap_object()? })
        }
    }

    impl JString {
        pub fn to_rust_string(&self) -> String {
            string_obj_to_string(self.normal_object.clone().into())
        }

        pub fn from_rust(jvm: &JVMState, int_state: &mut InterpreterStateGuard, rust_str: String) -> Result<JString, WasException> {
            let string_class = check_initing_or_inited_class(jvm, int_state, ClassName::string().into()).unwrap();//todo replace these unwraps
            push_new_object(jvm, int_state, &string_class);
            let string_object = int_state.pop_current_operand_stack();

            let vec1 = rust_str.chars().map(|c| JavaValue::Char(c as u16)).collect::<Vec<JavaValue>>();
            let array_object = ArrayObject {
                elems: UnsafeCell::new(vec1),
                elem_type: ClassName::string().into(),
                monitor: jvm.thread_state.new_monitor("monitor for a string".to_string()),
            };
            //todo what about check_inited_class for this array type
            let array = JavaValue::Object(Some(Arc::new(Object::Array(array_object))));
            run_constructor(jvm, int_state, string_class, vec![string_object.clone(), array],
                            "([C)V".to_string())?;
            Ok(string_object.cast_string().expect("error creating string"))
        }

        pub fn intern(&self, jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> Result<JString, WasException> {
            int_state.push_current_operand_stack(self.clone().java_value());
            let string_class = check_initing_or_inited_class(jvm, int_state, ClassName::string().into())?;
            run_static_or_virtual(
                jvm,
                int_state,
                &string_class,
                "intern".to_string(),
                "()Ljava/lang/String;".to_string(),
            )?;
            Ok(int_state.pop_current_operand_stack().cast_string().expect("error interning strinng"))
        }

        pub fn value(&self) -> Vec<jchar> {
            let mut res = vec![];
            for elem in self.normal_object.lookup_field("value").unwrap_array().unwrap_mut() {
                res.push(elem.unwrap_char())
            }
            res
        }

        pub fn length(&self, jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> Result<jint, WasException> {
            int_state.push_current_operand_stack(self.clone().java_value());
            let string_class = check_initing_or_inited_class(jvm, int_state, ClassName::string().into())?;
            run_static_or_virtual(
                jvm,
                int_state,
                &string_class,
                "length".to_string(),
                "()I".to_string(),
            )?;
            Ok(int_state.pop_current_operand_stack().unwrap_int())
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
    use crate::class_loading::assert_inited_or_initing_class;
    use crate::interpreter::WasException;
    use crate::interpreter_util::{push_new_object, run_constructor};
    use crate::java::lang::class_loader::ClassLoader;
    use crate::java::lang::string::JString;
    use crate::java::lang::thread_group::JThreadGroup;
    use crate::java_values::{NormalObject, Object};
    use crate::java_values::JavaValue;
    use crate::runtime_class::RuntimeClass;
    use crate::threading::{JavaThread, JavaThreadId};
    use crate::utils::run_static_or_virtual;

    #[derive(Debug, Clone)]
    pub struct JThread {
        normal_object: Arc<Object>
    }

    impl JavaValue {
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

        pub fn run(&self, jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> Result<(), WasException> {
            let thread_class = self.normal_object.unwrap_normal_object().class_pointer.clone();
            int_state.push_current_operand_stack(JavaValue::Object(self.normal_object.clone().into()));
            run_static_or_virtual(jvm, int_state, &thread_class, "run".to_string(), "()V".to_string())
        }

        pub fn exit(&self, jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> Result<(), WasException> {
            let thread_class = self.normal_object.unwrap_normal_object().class_pointer.clone();
            int_state.push_current_operand_stack(JavaValue::Object(self.normal_object.clone().into()));
            run_static_or_virtual(jvm, int_state, &thread_class, "exit".to_string(), "()V".to_string())
        }

        pub fn name(&self) -> JString {
            self.normal_object.lookup_field("name").cast_string().expect("threads are known to have nonnull names")
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


        pub fn new(jvm: &JVMState, int_state: &mut InterpreterStateGuard, thread_group: JThreadGroup, thread_name: String) -> Result<JThread, WasException> {
            let thread_class = assert_inited_or_initing_class(jvm, int_state, ClassName::thread().into());
            push_new_object(jvm, int_state, &thread_class);
            let thread_object = int_state.pop_current_operand_stack();
            let thread_name = JString::from_rust(jvm, int_state, thread_name)?;
            run_constructor(jvm, int_state, thread_class, vec![thread_object.clone(), thread_group.java_value(), thread_name.java_value()],
                            "(Ljava/lang/ThreadGroup;Ljava/lang/String;)V".to_string())?;
            Ok(thread_object.cast_thread())
        }

        pub fn get_java_thread(&self, jvm: &JVMState) -> Arc<JavaThread> {
            self.try_get_java_thread(jvm).unwrap()
        }

        pub fn try_get_java_thread(&self, jvm: &JVMState) -> Option<Arc<JavaThread>> {
            let tid = self.tid();
            jvm.thread_state.try_get_thread_by_tid(tid)
        }

        pub fn is_alive(&self, jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> Result<jboolean, WasException> {
            let thread_class = assert_inited_or_initing_class(jvm, int_state, ClassName::thread().into());
            int_state.push_current_operand_stack(self.clone().java_value());
            run_static_or_virtual(
                jvm,
                int_state,
                &thread_class,
                "isAlive".to_string(),
                "()Z".to_string(),
            )?;
            Ok(int_state.pop_current_operand_stack()
                .unwrap_boolean())
        }


        pub fn get_context_class_loader(&self, jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> Result<Option<ClassLoader>, WasException> {
            let thread_class = assert_inited_or_initing_class(jvm, int_state, ClassName::thread().into());
            int_state.push_current_operand_stack(self.clone().java_value());
            run_static_or_virtual(
                jvm,
                int_state,
                &thread_class,
                "getContextClassLoader".to_string(),
                "()Ljava/lang/ClassLoader;".to_string(),
            )?;
            let res = int_state.pop_current_operand_stack();
            if res.unwrap_object().is_none() {
                return Ok(None);
            }
            Ok(res.cast_class_loader().into())
        }

        pub fn get_inherited_access_control_context(&self) -> JThread {
            self.normal_object.lookup_field("inheritedAccessControlContext").cast_thread()
        }

        as_object_or_java_value!();
    }
}

pub mod thread_group {
    use std::sync::Arc;

    use jvmti_jni_bindings::{jboolean, jint};
    use rust_jvm_common::classnames::ClassName;

    use crate::{InterpreterStateGuard, JVMState};
    use crate::interpreter::WasException;
    use crate::interpreter_util::{push_new_object, run_constructor};
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
                    if normal_object.class_pointer.view().name() == ClassName::thread_group().into() {
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
                    int_state: &mut InterpreterStateGuard, thread_group_class: Arc<RuntimeClass>) -> Result<JThreadGroup, WasException> {
            push_new_object(jvm, int_state, &thread_group_class);
            let thread_group_object = int_state.pop_current_operand_stack();
            run_constructor(jvm, int_state, thread_group_class, vec![thread_group_object.clone()],
                            "()V".to_string())?;
            Ok(thread_group_object.cast_thread_group())
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
            self.normal_object.lookup_field("name").cast_string().expect("thread group null name")
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

    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::interpreter_util::{push_new_object, run_constructor};
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

        pub fn new(jvm: &JVMState, int_state: &mut InterpreterStateGuard, class: JString) -> Result<ClassNotFoundException, WasException> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, ClassName::Str("java/lang/ClassNotFoundException".to_string()).into())?;
            push_new_object(jvm, int_state, &class_not_found_class);
            let this = int_state.pop_current_operand_stack();
            run_constructor(jvm, int_state, class_not_found_class, vec![this.clone(), class.java_value()],
                            "(Ljava/lang/String;)V".to_string())?;
            Ok(this.cast_class_not_found_exception())
        }
    }
}

pub mod null_pointer_exception {
    use std::sync::Arc;

    use rust_jvm_common::classnames::ClassName;

    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::interpreter_util::{push_new_object, run_constructor};
    use crate::java::lang::string::JString;
    use crate::java_values::{JavaValue, Object};
    use crate::jvm_state::JVMState;

    pub struct NullPointerException {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_null_pointer_exception(&self) -> NullPointerException {
            NullPointerException { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl NullPointerException {
        as_object_or_java_value!();

        pub fn new(jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> Result<NullPointerException, WasException> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, ClassName::Str("java/lang/NullPointerException".to_string()).into())?;
            push_new_object(jvm, int_state, &class_not_found_class);
            let this = int_state.pop_current_operand_stack();
            let message = JString::from_rust(jvm, int_state, "This jvm doesn't believe in helpful null pointer messages so you get this instead".to_string())?;
            run_constructor(jvm, int_state, class_not_found_class, vec![this.clone(), message.java_value()],
                            "(Ljava/lang/String;)V".to_string())?;
            Ok(this.cast_null_pointer_exception())
        }
    }
}


pub mod array_out_of_bounds_exception {
    use std::sync::Arc;

    use jvmti_jni_bindings::jint;
    use rust_jvm_common::classnames::ClassName;

    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::interpreter_util::{push_new_object, run_constructor};
    use crate::java_values::{JavaValue, Object};
    use crate::jvm_state::JVMState;

    pub struct ArrayOutOfBoundsException {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_array_out_of_bounds_exception(&self) -> ArrayOutOfBoundsException {
            ArrayOutOfBoundsException { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl ArrayOutOfBoundsException {
        as_object_or_java_value!();

        pub fn new(jvm: &JVMState, int_state: &mut InterpreterStateGuard, index: jint) -> Result<ArrayOutOfBoundsException, WasException> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, ClassName::Str("java/lang/ArrayOutOfBoundsException".to_string()).into())?;
            push_new_object(jvm, int_state, &class_not_found_class);
            let this = int_state.pop_current_operand_stack();
            run_constructor(jvm, int_state, class_not_found_class, vec![this.clone(), JavaValue::Int(index)],
                            "(I)V".to_string())?;
            Ok(this.cast_array_out_of_bounds_exception())
        }
    }
}


pub mod illegal_argument_exception {
    use std::sync::Arc;

    use rust_jvm_common::classnames::ClassName;

    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::interpreter_util::{push_new_object, run_constructor};
    use crate::java_values::{JavaValue, Object};
    use crate::jvm_state::JVMState;

    pub struct IllegalArgumentException {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_illegal_argument_exception(&self) -> IllegalArgumentException {
            IllegalArgumentException { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl IllegalArgumentException {
        as_object_or_java_value!();

        pub fn new(jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> Result<IllegalArgumentException, WasException> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, ClassName::Str("java/lang/IllegalArgumentException".to_string()).into())?;
            push_new_object(jvm, int_state, &class_not_found_class);
            let this = int_state.pop_current_operand_stack();
            run_constructor(jvm, int_state, class_not_found_class, vec![this.clone()],
                            "()V".to_string())?;
            Ok(this.cast_illegal_argument_exception())
        }
    }
}

pub mod long {
    use std::sync::Arc;

    use jvmti_jni_bindings::jlong;
    use rust_jvm_common::classnames::ClassName;

    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::interpreter_util::{push_new_object, run_constructor};
    use crate::java_values::{JavaValue, Object};
    use crate::jvm_state::JVMState;

    pub struct Long {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_long(&self) -> Long {
            Long { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl Long {
        as_object_or_java_value!();

        pub fn new(jvm: &JVMState, int_state: &mut InterpreterStateGuard, param: jlong) -> Result<Long, WasException> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, ClassName::Str("java/lang/Long".to_string()).into())?;
            push_new_object(jvm, int_state, &class_not_found_class);
            let this = int_state.pop_current_operand_stack();
            run_constructor(jvm, int_state, class_not_found_class, vec![this.clone(), JavaValue::Long(param)],
                            "(J)V".to_string())?;
            Ok(this.cast_long())
        }
    }
}

pub mod int {
    use std::sync::Arc;

    use jvmti_jni_bindings::jint;
    use rust_jvm_common::classnames::ClassName;

    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::interpreter_util::{push_new_object, run_constructor};
    use crate::java_values::{JavaValue, Object};
    use crate::jvm_state::JVMState;

    pub struct Int {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_int(&self) -> Int {
            Int { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl Int {
        as_object_or_java_value!();

        pub fn new(jvm: &JVMState, int_state: &mut InterpreterStateGuard, param: jint) -> Result<Int, WasException> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, ClassName::Str("java/lang/Int".to_string()).into())?;
            push_new_object(jvm, int_state, &class_not_found_class);
            let this = int_state.pop_current_operand_stack();
            run_constructor(jvm, int_state, class_not_found_class, vec![this.clone(), JavaValue::Int(param)],
                            "(I)V".to_string())?;
            Ok(this.cast_int())
        }
    }
}

pub mod short {
    use std::sync::Arc;

    use jvmti_jni_bindings::jshort;
    use rust_jvm_common::classnames::ClassName;

    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::interpreter_util::{push_new_object, run_constructor};
    use crate::java_values::{JavaValue, Object};
    use crate::jvm_state::JVMState;

    pub struct Short {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_short(&self) -> Short {
            Short { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl Short {
        as_object_or_java_value!();

        pub fn new(jvm: &JVMState, int_state: &mut InterpreterStateGuard, param: jshort) -> Result<Short, WasException> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, ClassName::Str("java/lang/Short".to_string()).into())?;
            push_new_object(jvm, int_state, &class_not_found_class);
            let this = int_state.pop_current_operand_stack();
            run_constructor(jvm, int_state, class_not_found_class, vec![this.clone(), JavaValue::Short(param)],
                            "(S)V".to_string())?;
            Ok(this.cast_short())
        }
    }
}

pub mod byte {
    use std::sync::Arc;

    use jvmti_jni_bindings::jbyte;
    use rust_jvm_common::classnames::ClassName;

    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::interpreter_util::{push_new_object, run_constructor};
    use crate::java_values::{JavaValue, Object};
    use crate::jvm_state::JVMState;

    pub struct Byte {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_byte(&self) -> Byte {
            Byte { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl Byte {
        as_object_or_java_value!();

        pub fn new(jvm: &JVMState, int_state: &mut InterpreterStateGuard, param: jbyte) -> Result<Byte, WasException> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, ClassName::Str("java/lang/Byte".to_string()).into())?;
            push_new_object(jvm, int_state, &class_not_found_class);
            let this = int_state.pop_current_operand_stack();
            run_constructor(jvm, int_state, class_not_found_class, vec![this.clone(), JavaValue::Byte(param)],
                            "(B)V".to_string())?;
            Ok(this.cast_byte())
        }
    }
}

pub mod boolean {
    use std::sync::Arc;

    use jvmti_jni_bindings::jboolean;
    use rust_jvm_common::classnames::ClassName;

    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::interpreter_util::{push_new_object, run_constructor};
    use crate::java_values::{JavaValue, Object};
    use crate::jvm_state::JVMState;

    pub struct Boolean {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_boolean(&self) -> Boolean {
            Boolean { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl Boolean {
        as_object_or_java_value!();

        pub fn new(jvm: &JVMState, int_state: &mut InterpreterStateGuard, param: jboolean) -> Result<Boolean, WasException> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, ClassName::Str("java/lang/Boolean".to_string()).into())?;
            push_new_object(jvm, int_state, &class_not_found_class);
            let this = int_state.pop_current_operand_stack();
            run_constructor(jvm, int_state, class_not_found_class, vec![this.clone(), JavaValue::Boolean(param)],
                            "(Z)V".to_string())?;
            Ok(this.cast_boolean())
        }
    }
}

pub mod char {
    use std::sync::Arc;

    use jvmti_jni_bindings::jchar;
    use rust_jvm_common::classnames::ClassName;

    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::interpreter_util::{push_new_object, run_constructor};
    use crate::java_values::{JavaValue, Object};
    use crate::jvm_state::JVMState;

    pub struct Char {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_char(&self) -> Char {
            Char { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl Char {
        as_object_or_java_value!();

        pub fn new(jvm: &JVMState, int_state: &mut InterpreterStateGuard, param: jchar) -> Result<Char, WasException> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, ClassName::Str("java/lang/Char".to_string()).into())?;
            push_new_object(jvm, int_state, &class_not_found_class);
            let this = int_state.pop_current_operand_stack();
            run_constructor(jvm, int_state, class_not_found_class, vec![this.clone(), JavaValue::Char(param)],
                            "(C)V".to_string())?;
            Ok(this.cast_char())
        }
    }
}

pub mod float {
    use std::sync::Arc;

    use jvmti_jni_bindings::jfloat;
    use rust_jvm_common::classnames::ClassName;

    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::interpreter_util::{push_new_object, run_constructor};
    use crate::java_values::{JavaValue, Object};
    use crate::jvm_state::JVMState;

    pub struct Float {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_float(&self) -> Float {
            Float { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl Float {
        as_object_or_java_value!();

        pub fn new(jvm: &JVMState, int_state: &mut InterpreterStateGuard, param: jfloat) -> Result<Float, WasException> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, ClassName::Str("java/lang/Float".to_string()).into())?;
            push_new_object(jvm, int_state, &class_not_found_class);
            let this = int_state.pop_current_operand_stack();
            run_constructor(jvm, int_state, class_not_found_class, vec![this.clone(), JavaValue::Float(param)],
                            "(F)V".to_string())?;
            Ok(this.cast_float())
        }
    }
}

pub mod double {
    use std::sync::Arc;

    use jvmti_jni_bindings::jdouble;
    use rust_jvm_common::classnames::ClassName;

    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::interpreter_util::{push_new_object, run_constructor};
    use crate::java_values::{JavaValue, Object};
    use crate::jvm_state::JVMState;

    pub struct Double {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_double(&self) -> Double {
            Double { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl Double {
        as_object_or_java_value!();

        pub fn new(jvm: &JVMState, int_state: &mut InterpreterStateGuard, param: jdouble) -> Result<Double, WasException> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, ClassName::Str("java/lang/Double".to_string()).into())?;
            push_new_object(jvm, int_state, &class_not_found_class);
            let this = int_state.pop_current_operand_stack();
            run_constructor(jvm, int_state, class_not_found_class, vec![this.clone(), JavaValue::Double(param)],
                            "(D)V".to_string())?;
            Ok(this.cast_double())
        }
    }
}


pub mod system;
pub mod reflect;