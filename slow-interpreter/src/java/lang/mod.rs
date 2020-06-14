pub mod invoke;


pub mod member_name {
    use crate::java_values::{JavaValue, Object};
    use crate::java::lang::string::JString;
    use crate::instructions::invoke::native::mhn_temp::run_static_or_virtual;
    use crate::{JVMState, StackEntry};

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
        pub fn get_name(&self, jvm: &JVMState, frame: &StackEntry) -> JString {
            let member_name_class = check_inited_class(jvm, &ClassName::member_name().into(), frame.class_pointer.loader(jvm).clone());
            frame.push(JavaValue::Object(self.normal_object.clone().into()));
            run_static_or_virtual(jvm, &member_name_class, "getName".to_string(), "()Ljava/lang/String;".to_string());
            frame.pop().cast_string()
        }

        pub fn clazz(&self) -> JClass {
            self.normal_object.unwrap_normal_object().fields.borrow().get("clazz").unwrap().cast_class()
        }

        pub fn get_method_type(&self, jvm: &JVMState, frame: &StackEntry) -> MethodType {
            let member_name_class = check_inited_class(jvm, &ClassName::member_name().into(), frame.class_pointer.loader(jvm).clone());
            frame.push(JavaValue::Object(self.normal_object.clone().into()));
            run_static_or_virtual(jvm, &member_name_class, "getMethodType".to_string(), "()Ljava/lang/invoke/MethodType;".to_string());
            frame.pop().cast_method_type()
        }

        pub fn get_field_type(&self, jvm: &JVMState, frame: &StackEntry) -> JClass {
            let member_name_class = check_inited_class(jvm, &ClassName::member_name().into(), frame.class_pointer.loader(jvm).clone());
            frame.push(JavaValue::Object(self.normal_object.clone().into()));
            run_static_or_virtual(jvm, &member_name_class, "getFieldType".to_string(), "()Ljava/lang/Class;".to_string());
            frame.pop().cast_class()
        }
    }
}

pub mod class {
    use crate::java_values::{JavaValue, Object};
    use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
    use std::sync::Arc;
    use crate::java::lang::class_loader::ClassLoader;
    use crate::instructions::invoke::native::mhn_temp::run_static_or_virtual;

    use crate::{StackEntry, JVMState};
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

        pub fn get_class_loader(&self, state: &JVMState, frame: &StackEntry) -> Option<ClassLoader> {
            frame.push(JavaValue::Object(self.normal_object.clone().into()));
            run_static_or_virtual(
                state,
                &self.normal_object.unwrap_normal_object().class_pointer,
                "getClassLoader".to_string(),
                "()Ljava/lang/ClassLoader;".to_string(),
            );
            frame.pop()
                .unwrap_object()
                .map(|cl| JavaValue::Object(cl.into()).cast_class_loader())
        }

        pub fn from_name(jvm: &JVMState, frame: &StackEntry, name: ClassName) -> JClass {
            let type_ = PTypeView::Ref(ReferenceTypeView::Class(name));
            let loader_arc = frame.class_pointer.loader(jvm).clone();
            JavaValue::Object(get_or_create_class_object(jvm, &type_, frame.clone(), loader_arc).into()).cast_class()
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
    use crate::{JVMState, StackEntry};


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

        pub fn from(state: &JVMState, current_frame: &StackEntry, rust_str: String) -> JString {
            create_string_on_stack(state, rust_str);
            current_frame.pop().cast_string()
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
        pub fn from(_state: &JVMState, _current_frame: &StackEntry, _i: jint) -> Integer {
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
    use crate::java_values::Object;
    use std::sync::Arc;
    use crate::java_values::JavaValue;
    use crate::JVMState;
    use crate::stack_entry::StackEntry;
    use crate::instructions::invoke::native::mhn_temp::run_static_or_virtual;
    use crate::interpreter_util::check_inited_class;
    use rust_jvm_common::classnames::ClassName;
    use crate::java::lang::string::JString;

    #[derive(Debug,Clone)]
    pub struct JThread {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_thread(&self) -> JThread {
            JThread { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl JThread {
        pub fn tid(&self)-> i64{
            self.normal_object.unwrap_normal_object().fields.borrow().get("tid").unwrap().unwrap_long()
        }

        pub fn run(&self, jvm : &JVMState, frame: &StackEntry){
            let thread_class = check_inited_class(jvm,&ClassName::thread().into(),frame.class_pointer.loader(jvm).clone());
            frame.push(JavaValue::Object(self.normal_object.clone().into()));
            run_static_or_virtual(jvm,&thread_class,"run".to_string(),"()V".to_string());

        }

        pub fn name(&self) -> JString{
            self.normal_object.lookup_field("name").cast_string()
        }

        pub fn priority(&self) -> i32{
            self.normal_object.lookup_field("priority").unwrap_int()
        }

        pub fn daemon(&self) -> bool{
            self.normal_object.lookup_field("daemon").unwrap_int() != 0
        }

        as_object_or_java_value!();
    }
}


pub mod system {
    use std::sync::Arc;
    use crate::java::util::properties::Properties;
    use crate::JVMState;
    use crate::interpreter_util::check_inited_class;
    use rust_jvm_common::classnames::ClassName;
    use std::borrow::Borrow;
    use crate::java_values::{Object, JavaValue};

    pub struct System {
        normal_object: Arc<Object>
    }

    impl System {
        pub fn props(jvm : &JVMState) -> Properties{
            let system_class = check_inited_class(jvm,&ClassName::system().into(),jvm.bootstrap_loader.clone());
            let prop_jv = system_class.static_vars().borrow().get("props").unwrap().clone();
            prop_jv.cast_properties()
        }

        as_object_or_java_value!();
    }
}

pub mod reflect;