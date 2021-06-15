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
    pub struct Throwable<'gc_life> {
        pub(crate) normal_object: Arc<Object<'gc_life>>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_throwable(&self) -> Throwable {
            Throwable { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> Throwable<'gc_life> {
        pub fn print_stack_trace<'l, 'k : 'l>(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>) -> Result<(), WasException> {
            let throwable_class = check_initing_or_inited_class(jvm, int_state, ClassName::throwable().into()).expect("Throwable isn't inited?");
            int_state.push_current_operand_stack(JavaValue::Object(todo!()/*self.normal_object.clone().into()*/));
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
    pub struct StackTraceElement<'gc_life> {
        normal_object: Arc<Object<'gc_life>>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_stack_trace_element(&self) -> StackTraceElement<'gc_life> {
            StackTraceElement { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> StackTraceElement<'gc_life> {
        pub fn new<'l, 'k : 'l>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>, declaring_class: JString<'gc_life>, method_name: JString<'gc_life>, file_name: JString<'gc_life>, line_number: jint) -> Result<StackTraceElement<'gc_life>, WasException> {
            let class_ = check_initing_or_inited_class(jvm, int_state, ClassName::new("java/lang/StackTraceElement").into())?;
            push_new_object(jvm, int_state, &class_);
            let res = int_state.pop_current_operand_stack(ClassName::object().into());
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

    use classfile_view::view::ptype_view::PTypeView;
    use jvmti_jni_bindings::jint;
    use rust_jvm_common::classnames::ClassName;
    use type_safe_proc_macro_utils::getter_gen;

    use crate::{InterpreterStateGuard, JVMState};
    use crate::class_loading::{assert_inited_or_initing_class, check_initing_or_inited_class};
    use crate::interpreter::WasException;
    use crate::interpreter_util::{push_new_object, run_constructor};
    use crate::java::lang::class::JClass;
    use crate::java::lang::invoke::method_type::MethodType;
    use crate::java::lang::reflect::constructor::Constructor;
    use crate::java::lang::reflect::field::Field;
    use crate::java::lang::reflect::method::Method;
    use crate::java::lang::string::JString;
    use crate::java_values::{JavaValue, Object};
    use crate::utils::run_static_or_virtual;

    #[derive(Clone, Debug)]
    pub struct MemberName<'gc_life> {
        normal_object: Arc<Object<'gc_life>>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_member_name(&self) -> MemberName {
            MemberName { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> MemberName<'gc_life> {
        // private Class<?> clazz;
        // private String name;
        // private Object type;
        // private int flags;
        pub fn get_name_func<'l, 'k : 'l>(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>) -> Result<Option<JString>, WasException> {
            let member_name_class = assert_inited_or_initing_class(jvm, ClassName::member_name().into());
            int_state.push_current_operand_stack(JavaValue::Object(todo!()/*self.normal_object.clone().into()*/));
            run_static_or_virtual(jvm, int_state, &member_name_class, "getName".to_string(), "()Ljava/lang/String;".to_string())?;
            Ok(int_state.pop_current_operand_stack(ClassName::string().into()).cast_string())
        }

        pub fn is_static<'l, 'k : 'l>(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>) -> Result<bool, WasException> {
            let member_name_class = assert_inited_or_initing_class(jvm, ClassName::member_name().into());
            int_state.push_current_operand_stack(JavaValue::Object(todo!()/*self.normal_object.clone().into()*/));
            run_static_or_virtual(jvm, int_state, &member_name_class, "isStatic".to_string(), "()Z".to_string())?;
            Ok(int_state.pop_current_operand_stack(PTypeView::BooleanType).unwrap_boolean() != 0)
        }

        pub fn get_name_or_null(&self) -> Option<JString<'gc_life>> {
            let str_jvalue = self.normal_object.lookup_field("name");
            if str_jvalue.unwrap_object().is_none() {
                None
            } else {
                str_jvalue.cast_string().into()
            }
        }

        pub fn get_name(&self) -> JString<'gc_life> {
            self.get_name_or_null().unwrap()
        }


        pub fn set_name(&self, new_val: JString<'gc_life>) {
            self.normal_object.unwrap_normal_object().set_var_top_level("name", new_val.java_value());
        }

        pub fn get_clazz_or_null(&self) -> Option<JClass<'gc_life>> {
            let possibly_null = self.normal_object.unwrap_normal_object().get_var_top_level("clazz").clone();
            if possibly_null.unwrap_object().is_none() {
                None
            } else {
                possibly_null.cast_class().into()
            }
        }

        pub fn get_clazz(&self) -> JClass<'gc_life> {
            self.get_clazz_or_null().unwrap()
        }

        pub fn set_clazz(&self, new_val: JClass<'gc_life>) {
            self.normal_object.unwrap_normal_object().set_var_top_level("clazz".to_string(), new_val.java_value());
        }

        pub fn set_type(&self, new_val: MethodType<'gc_life>) {
            self.normal_object.unwrap_normal_object().set_var_top_level("type".to_string(), new_val.java_value());
        }


        pub fn get_type(&self) -> JavaValue<'gc_life> {
            self.normal_object.unwrap_normal_object().get_var_top_level("type").clone()
        }

        pub fn set_flags(&self, new_val: jint) {
            self.normal_object.unwrap_normal_object().set_var_top_level("flags".to_string(), JavaValue::Int(new_val));
        }


        getter_gen!(flags,jint,unwrap_int);

        pub fn set_resolution(&self, new_val: JavaValue<'gc_life>) {
            self.normal_object.unwrap_normal_object().set_var_top_level("resolution".to_string(), new_val);
        }

        pub fn get_resolution(&self) -> JavaValue<'gc_life> {
            self.normal_object.unwrap_normal_object().get_var_top_level(&"resolution".to_string()).clone()
        }

        pub fn clazz(&self) -> Option<JClass<'gc_life>> {
            self.normal_object.unwrap_normal_object().get_var_top_level("clazz").cast_class()
        }

        pub fn get_method_type<'l, 'k : 'l>(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>) -> Result<MethodType, WasException> {
            let member_name_class = assert_inited_or_initing_class(jvm, ClassName::member_name().into());
            int_state.push_current_operand_stack(JavaValue::Object(todo!()/*self.normal_object.clone().into()*/));
            run_static_or_virtual(jvm, int_state, &member_name_class, "getMethodType".to_string(), "()Ljava/lang/invoke/MethodType;".to_string())?;
            Ok(int_state.pop_current_operand_stack(ClassName::method_type().into()).cast_method_type())
        }

        pub fn get_field_type<'l, 'k : 'l>(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>) -> Result<Option<JClass<'gc_life>>, WasException> {
            let member_name_class = assert_inited_or_initing_class(jvm, ClassName::member_name().into());
            int_state.push_current_operand_stack(JavaValue::Object(todo!()/*self.normal_object.clone().into()*/));
            run_static_or_virtual(jvm, int_state, &member_name_class, "getFieldType".to_string(), "()Ljava/lang/Class;".to_string())?;
            Ok(int_state.pop_current_operand_stack(ClassName::class().into()).cast_class())
        }

        pub fn new_from_field<'l, 'k : 'l>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>, field: Field<'gc_life>) -> Result<Self, WasException> {
            let member_class = check_initing_or_inited_class(jvm, int_state, ClassName::member_name().into())?;
            push_new_object(jvm, int_state, &member_class);
            let res = int_state.pop_current_operand_stack(ClassName::object().into());
            run_constructor(jvm, int_state, member_class, vec![res.clone(), field.java_value()], "(Ljava/lang/reflect/Field;)V".to_string())?;
            Ok(res.cast_member_name())
        }

        pub fn new_from_method<'l, 'k : 'l>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>, method: Method<'gc_life>) -> Result<Self, WasException> {
            let member_class = check_initing_or_inited_class(jvm, int_state, ClassName::member_name().into())?;
            push_new_object(jvm, int_state, &member_class);
            let res = int_state.pop_current_operand_stack(ClassName::object().into());
            run_constructor(jvm, int_state, member_class, vec![res.clone(), method.java_value()], "(Ljava/lang/reflect/Method;)V".to_string())?;
            Ok(res.cast_member_name())
        }


        pub fn new_from_constructor<'l, 'k : 'l>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>, constructor: Constructor<'gc_life>) -> Result<Self, WasException> {
            let member_class = check_initing_or_inited_class(jvm, int_state, ClassName::member_name().into())?;
            push_new_object(jvm, int_state, &member_class);
            let res = int_state.pop_current_operand_stack(ClassName::object().into());
            run_constructor(jvm, int_state, member_class, vec![res.clone(), constructor.java_value()], "(Ljava/lang/reflect/Constructor;)V".to_string())?;
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
    pub struct JClass<'gc_life> {
        normal_object: Arc<Object<'gc_life>>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_class(&self) -> Option<JClass<'gc_life>> {
            Some(JClass { normal_object: self.unwrap_object()? })
        }
    }

    impl<'gc_life> JClass<'gc_life> {
        pub fn as_type(&self, jvm: &'gc_life JVMState<'gc_life>) -> PTypeView {
            self.as_runtime_class(jvm).ptypeview()
        }

        pub fn as_runtime_class(&self, jvm: &'gc_life JVMState<'gc_life>) -> Arc<RuntimeClass<'gc_life>> {
            jvm.classes.read().unwrap().class_object_pool.get_by_left(&ByAddress(self.normal_object.clone())).unwrap().clone().0
        }

        pub fn get_class_loader<'l, 'k : 'l>(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>) -> Result<Option<ClassLoader>, WasException> {
            int_state.push_current_operand_stack(JavaValue::Object(todo!()/*self.normal_object.clone().into()*/));
            run_static_or_virtual(
                jvm,
                int_state,
                &self.normal_object.unwrap_normal_object().objinfo.class_pointer,
                "getClassLoader".to_string(),
                "()Ljava/lang/ClassLoader;".to_string(),
            )?;
            Ok(int_state.pop_current_operand_stack(ClassName::object().into())
                .unwrap_object()
                .map(|cl| todo!()/*/*JavaValue::Object(cl.into()*/).cast_class_loader()*/))
        }

        pub fn new_bootstrap_loader<'l, 'k : 'l>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>) -> Result<Self, WasException> {
            let class_class = check_initing_or_inited_class(jvm, int_state, ClassName::class().into())?;
            push_new_object(jvm, int_state, &class_class);
            let res = int_state.pop_current_operand_stack(ClassName::class().into());
            run_constructor(jvm, int_state, class_class, vec![res.clone(), JavaValue::Object(todo!()/*None*/)], "(Ljava/lang/ClassLoader;)V".to_string())?;
            Ok(res.cast_class().unwrap())
        }


        pub fn new<'l, 'k : 'l>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>, loader: ClassLoader<'gc_life>) -> Result<Self, WasException> {
            let class_class = check_initing_or_inited_class(jvm, int_state, ClassName::class().into())?;
            push_new_object(jvm, int_state, &class_class);
            let res = int_state.pop_current_operand_stack(ClassName::class().into());
            run_constructor(jvm, int_state, class_class, vec![res.clone(), loader.java_value()], "(Ljava/lang/ClassLoader;)V".to_string())?;
            Ok(res.cast_class().unwrap())
        }

        pub fn from_name(jvm: &'gc_life JVMState<'gc_life>, int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>, name: ClassName) -> JClass<'gc_life> {
            let type_ = PTypeView::Ref(ReferenceTypeView::Class(name));
            JavaValue::Object(todo!()/*get_or_create_class_object(jvm, type_, int_state).unwrap().into()*/).cast_class().unwrap()
        }

        pub fn from_type<'k: 'l, 'l>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>, ptype: PTypeView) -> Result<JClass<'gc_life>, WasException> {
            load_class_constant_by_type(jvm, int_state, ptype)?;
            let res = int_state.pop_current_operand_stack(ClassName::class().into()).unwrap_object();
            Ok(JavaValue::Object(todo!()/*res*/).cast_class().unwrap())
        }

        pub fn get_name<'l, 'k : 'l>(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>) -> Result<JString<'gc_life>, WasException> {
            int_state.push_current_operand_stack(self.clone().java_value());
            let class_class = check_initing_or_inited_class(jvm, int_state, ClassName::class().into()).unwrap();
            run_static_or_virtual(
                jvm,
                int_state,
                &class_class,
                "getName".to_string(),
                "()Ljava/lang/String;".to_string(),
            )?;
            Ok(int_state.pop_current_operand_stack(ClassName::string().into()).cast_string().expect("classes are known to have non-null names"))
        }

        pub fn set_name_(&self, name: JString<'gc_life>) {
            let normal_object = self.normal_object.unwrap_normal_object();
            normal_object.set_var_top_level("name".to_string(), name.java_value());
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
    pub struct ClassLoader<'gc_life> {
        normal_object: Arc<Object<'gc_life>>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_class_loader(&self) -> ClassLoader {
            ClassLoader { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> ClassLoader<'gc_life> {
        pub fn to_jvm_loader(&self, jvm: &'gc_life JVMState<'gc_life>) -> LoaderName {
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

        pub fn load_class<'l, 'k : 'l>(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>, name: JString<'gc_life>) -> Result<JClass<'gc_life>, WasException> {
            int_state.push_current_operand_stack(self.clone().java_value());
            int_state.push_current_operand_stack(name.java_value());
            let class_loader = assert_inited_or_initing_class(jvm, ClassName::classloader().into());
            run_static_or_virtual(
                jvm,
                int_state,
                &class_loader,
                "loadClass".to_string(),
                "(Ljava/lang/String;)Ljava/lang/Class;".to_string(),
            )?;
            assert!(int_state.throw().is_none());
            Ok(int_state.pop_current_operand_stack(ClassName::class().into()).cast_class().unwrap())
        }

        as_object_or_java_value!();
    }
}

pub mod string {
    use std::cell::UnsafeCell;
    use std::sync::Arc;

    use classfile_view::view::ptype_view::PTypeView;
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
    pub struct JString<'gc_life> {
        normal_object: Arc<Object<'gc_life>>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_string(&self) -> Option<JString<'gc_life>> {
            Some(JString { normal_object: self.unwrap_object()? })
        }
    }

    impl<'gc_life> JString<'gc_life> {
        pub fn to_rust_string(&self) -> String {
            string_obj_to_string(todo!()/*self.normal_object.clone().into()*/)
        }

        pub fn from_rust<'l, 'k : 'l>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>, rust_str: String) -> Result<JString<'gc_life>, WasException> {
            let string_class = check_initing_or_inited_class(jvm, int_state, ClassName::string().into()).unwrap();//todo replace these unwraps
            push_new_object(jvm, int_state, &string_class);
            let string_object = int_state.pop_current_operand_stack(ClassName::string().into());

            let vec1 = rust_str.chars().map(|c| JavaValue::Char(c as u16)).collect::<Vec<JavaValue<'gc_life>>>();
            let array_object = ArrayObject {
                elems: UnsafeCell::new(vec1),
                elem_type: PTypeView::CharType,
                monitor: jvm.thread_state.new_monitor("monitor for a string".to_string()),
            };
            //todo what about check_inited_class for this array type
            let array = JavaValue::Object(todo!()/*Some(Arc::new(Object::Array(array_object)))*/);
            run_constructor(jvm, int_state, string_class, vec![string_object.clone(), array],
                            "([C)V".to_string())?;
            Ok(string_object.cast_string().expect("error creating string"))
        }

        pub fn intern<'l, 'k : 'l>(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>) -> Result<JString, WasException> {
            int_state.push_current_operand_stack(self.clone().java_value());
            let string_class = check_initing_or_inited_class(jvm, int_state, ClassName::string().into())?;
            run_static_or_virtual(
                jvm,
                int_state,
                &string_class,
                "intern".to_string(),
                "()Ljava/lang/String;".to_string(),
            )?;
            Ok(int_state.pop_current_operand_stack(ClassName::string().into()).cast_string().expect("error interning strinng"))
        }

        pub fn value(&self) -> Vec<jchar> {
            let mut res = vec![];
            for elem in self.normal_object.lookup_field("value").unwrap_array().unwrap_mut() {
                res.push(elem.unwrap_char())
            }
            res
        }

        pub fn length<'l, 'k : 'l>(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>) -> Result<jint, WasException> {
            int_state.push_current_operand_stack(self.clone().java_value());
            let string_class = check_initing_or_inited_class(jvm, int_state, ClassName::string().into())?;
            run_static_or_virtual(
                jvm,
                int_state,
                &string_class,
                "length".to_string(),
                "()I".to_string(),
            )?;
            Ok(int_state.pop_current_operand_stack(ClassName::string().into()).unwrap_int())
        }

        as_object_or_java_value!();
    }
}

pub mod integer {
    use std::sync::Arc;

    use jvmti_jni_bindings::jint;

    use crate::{JVMState, StackEntry};
    use crate::java_values::{JavaValue, Object};

    pub struct Integer<'gc_life> {
        normal_object: Arc<Object<'gc_life>>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_integer(&self) -> Integer {
            Integer { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> Integer<'gc_life> {
        pub fn from(_state: &JVMState, _current_frame: &StackEntry, _i: jint) -> Integer<'gc_life> {
            unimplemented!()
        }

        pub fn value(&self) -> jint {
            self.normal_object.unwrap_normal_object().get_var_top_level("value").unwrap_int()
        }

        as_object_or_java_value!();
    }
}

pub mod object {
    use std::sync::Arc;

    use crate::java_values::JavaValue;
    use crate::java_values::Object;

    pub struct JObject<'gc_life> {
        normal_object: Arc<Object<'gc_life>>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_object(&self) -> JObject {
            JObject { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> JObject<'gc_life> {
        as_object_or_java_value!();
    }
}

pub mod thread {
    use std::sync::Arc;

    use classfile_view::view::ptype_view::PTypeView;
    use jvmti_jni_bindings::{jboolean, jint};
    use rust_jvm_common::classnames::ClassName;

    use crate::{InterpreterStateGuard, JVMState};
    use crate::class_loading::assert_inited_or_initing_class;
    use crate::interpreter::WasException;
    use crate::interpreter_util::{push_new_object, run_constructor};
    use crate::java::lang::class_loader::ClassLoader;
    use crate::java::lang::string::JString;
    use crate::java::lang::thread_group::JThreadGroup;
    use crate::java_values::{NormalObject, Object, ObjectFieldsAndClass};
    use crate::java_values::JavaValue;
    use crate::runtime_class::RuntimeClass;
    use crate::threading::{JavaThread, JavaThreadId};
    use crate::utils::run_static_or_virtual;

    #[derive(Debug, Clone)]
    pub struct JThread<'gc_life> {
        normal_object: Arc<Object<'gc_life>>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_thread(&self) -> JThread {
            JThread { normal_object: self.unwrap_object_nonnull() }
        }

        pub fn try_cast_thread(&self) -> Option<JThread<'gc_life>> {
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

    impl<'gc_life> JThread<'gc_life> {
        pub fn invalid_thread(jvm: &'gc_life JVMState<'gc_life>) -> JThread<'gc_life> {
            JThread {
                normal_object: Arc::new(Object::Object(NormalObject {
                    monitor: jvm.thread_state.new_monitor("invalid thread monitor".to_string()),

                    objinfo: ObjectFieldsAndClass {
                        fields: Default::default(),
                        class_pointer: Arc::new(RuntimeClass::Top),
                    },
                }))
            }
        }

        pub fn tid(&self, jvm: &'gc_life JVMState<'gc_life>) -> JavaThreadId {
            let thread_class = assert_inited_or_initing_class(jvm, ClassName::thread().into());
            self.normal_object.unwrap_normal_object().get_var(thread_class, "tid", PTypeView::LongType).unwrap_long()
        }

        pub fn run<'l, 'k : 'l>(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>) -> Result<(), WasException> {
            let thread_class = self.normal_object.unwrap_normal_object().objinfo.class_pointer.clone();
            int_state.push_current_operand_stack(JavaValue::Object(todo!()/*self.normal_object.clone().into()*/));
            run_static_or_virtual(jvm, int_state, &thread_class, "run".to_string(), "()V".to_string())
        }

        pub fn exit<'l, 'k : 'l>(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>) -> Result<(), WasException> {
            let thread_class = self.normal_object.unwrap_normal_object().objinfo.class_pointer.clone();
            int_state.push_current_operand_stack(JavaValue::Object(todo!()/*self.normal_object.clone().into()*/));
            run_static_or_virtual(jvm, int_state, &thread_class, "exit".to_string(), "()V".to_string())
        }

        pub fn name(&self, jvm: &'gc_life JVMState<'gc_life>) -> JString {
            let thread_class = assert_inited_or_initing_class(jvm, ClassName::thread().into());
            self.normal_object.unwrap_normal_object().get_var(thread_class, "name", ClassName::string().into()).cast_string().expect("threads are known to have nonnull names")
        }

        pub fn priority(&self, jvm: &'gc_life JVMState<'gc_life>) -> i32 {
            let thread_class = assert_inited_or_initing_class(jvm, ClassName::thread().into());
            self.normal_object.unwrap_normal_object().get_var(thread_class, "priority", PTypeView::IntType).unwrap_int()
        }

        pub fn set_priority(&self, priority: i32) {
            self.normal_object.unwrap_normal_object().set_var_top_level("priority".to_string(), JavaValue::Int(priority));
        }

        pub fn daemon(&self, jvm: &'gc_life JVMState<'gc_life>) -> bool {
            let thread_class = assert_inited_or_initing_class(jvm, ClassName::thread().into());
            self.normal_object.unwrap_normal_object().get_var(thread_class, "daemon", PTypeView::BooleanType).unwrap_int() != 0
        }

        pub fn set_thread_status(&self, jvm: &'gc_life JVMState<'gc_life>, thread_status: jint) {
            let thread_class = assert_inited_or_initing_class(jvm, ClassName::thread().into());
            self.normal_object.unwrap_normal_object().set_var(thread_class, "threadStatus".to_string(), JavaValue::Int(thread_status), PTypeView::IntType);
        }


        pub fn new<'l, 'k : 'l>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>, thread_group: JThreadGroup<'gc_life>, thread_name: String) -> Result<JThread<'gc_life>, WasException> {
            let thread_class = assert_inited_or_initing_class(jvm, ClassName::thread().into());
            push_new_object(jvm, int_state, &thread_class);
            let thread_object = int_state.pop_current_operand_stack(ClassName::thread().into());
            let thread_name = JString::from_rust(jvm, int_state, thread_name)?;
            run_constructor(jvm, int_state, thread_class, vec![thread_object.clone(), thread_group.java_value(), thread_name.java_value()],
                            "(Ljava/lang/ThreadGroup;Ljava/lang/String;)V".to_string())?;
            Ok(thread_object.cast_thread())
        }

        pub fn get_java_thread(&self, jvm: &'gc_life JVMState<'gc_life>) -> Arc<JavaThread<'gc_life>> {
            self.try_get_java_thread(jvm).unwrap()
        }

        pub fn try_get_java_thread(&self, jvm: &'gc_life JVMState<'gc_life>) -> Option<Arc<JavaThread<'gc_life>>> {
            let tid = self.tid(jvm);
            jvm.thread_state.try_get_thread_by_tid(tid)
        }

        pub fn is_alive<'l, 'k : 'l>(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>) -> Result<jboolean, WasException> {
            let thread_class = assert_inited_or_initing_class(jvm, ClassName::thread().into());
            int_state.push_current_operand_stack(self.clone().java_value());
            run_static_or_virtual(
                jvm,
                int_state,
                &thread_class,
                "isAlive".to_string(),
                "()Z".to_string(),
            )?;
            Ok(int_state.pop_current_operand_stack(PTypeView::BooleanType)
                .unwrap_boolean())
        }


        pub fn get_context_class_loader<'l, 'k : 'l>(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>) -> Result<Option<ClassLoader>, WasException> {
            let thread_class = assert_inited_or_initing_class(jvm, ClassName::thread().into());
            int_state.push_current_operand_stack(self.clone().java_value());
            run_static_or_virtual(
                jvm,
                int_state,
                &thread_class,
                "getContextClassLoader".to_string(),
                "()Ljava/lang/ClassLoader;".to_string(),
            )?;
            let res = int_state.pop_current_operand_stack(ClassName::classloader().into());
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
    pub struct JThreadGroup<'gc_life> {
        normal_object: Arc<Object<'gc_life>>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_thread_group(&self) -> JThreadGroup {
            JThreadGroup { normal_object: self.unwrap_object_nonnull() }
        }

        pub fn try_cast_thread_group(&self) -> Option<JThreadGroup> {
            match self.try_unwrap_normal_object() {
                Some(normal_object) => {
                    if normal_object.objinfo.class_pointer.view().name() == ClassName::thread_group().into() {
                        return JThreadGroup { normal_object: self.unwrap_object_nonnull() }.into();
                    }
                    None
                }
                None => None
            }
        }
    }

    impl<'gc_life> JThreadGroup<'gc_life> {
        pub fn init<'l, 'k : 'l>(jvm: &'gc_life JVMState<'gc_life>,
                                 int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>, thread_group_class: Arc<RuntimeClass<'gc_life>>) -> Result<JThreadGroup<'gc_life>, WasException> {
            push_new_object(jvm, int_state, &thread_group_class);
            let thread_group_object = int_state.pop_current_operand_stack(ClassName::thread().into());
            run_constructor(jvm, int_state, thread_group_class, vec![thread_group_object.clone()],
                            "()V".to_string())?;
            Ok(thread_group_object.cast_thread_group())
        }

        pub fn threads(&self) -> Vec<Option<JThread<'gc_life>>> {
            unsafe {
                self.normal_object.lookup_field("threads").unwrap_array().elems.get().as_ref().unwrap().iter().map(|thread|
                    {
                        match thread.unwrap_object() {
                            None => None,
                            Some(t) => JavaValue::Object(todo!()/*t.into()*/).cast_thread().into(),
                        }
                    }
                ).collect()
            }
        }

        pub fn threads_non_null(&self) -> Vec<JThread<'gc_life>> {
            self.threads().into_iter().flatten().collect()
        }

        pub fn name(&self) -> JString<'gc_life> {
            self.normal_object.lookup_field("name").cast_string().expect("thread group null name")
        }

        pub fn daemon(&self) -> jboolean {
            self.normal_object.lookup_field("daemon").unwrap_boolean()
        }

        pub fn max_priority(&self) -> jint {
            self.normal_object.lookup_field("maxPriority").unwrap_int()
        }

        pub fn parent(&self) -> Option<JThreadGroup<'gc_life>> {
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

    pub struct ClassNotFoundException<'gc_life> {
        normal_object: Arc<Object<'gc_life>>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_class_not_found_exception(&self) -> ClassNotFoundException {
            ClassNotFoundException { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> ClassNotFoundException<'gc_life> {
        as_object_or_java_value!();

        pub fn new<'l, 'k : 'l>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>, class: JString<'gc_life>) -> Result<ClassNotFoundException<'gc_life>, WasException> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, ClassName::Str("java/lang/ClassNotFoundException".to_string()).into())?;
            push_new_object(jvm, int_state, &class_not_found_class);
            let this = int_state.pop_current_operand_stack(ClassName::object().into());
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

    pub struct NullPointerException<'gc_life> {
        normal_object: Arc<Object<'gc_life>>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_null_pointer_exception(&self) -> NullPointerException {
            NullPointerException { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> NullPointerException<'gc_life> {
        as_object_or_java_value!();

        pub fn new<'l, 'k : 'l>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>) -> Result<NullPointerException<'gc_life>, WasException> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, ClassName::Str("java/lang/NullPointerException".to_string()).into())?;
            push_new_object(jvm, int_state, &class_not_found_class);
            let this = int_state.pop_current_operand_stack(ClassName::object().into());
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

    pub struct ArrayOutOfBoundsException<'gc_life> {
        normal_object: Arc<Object<'gc_life>>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_array_out_of_bounds_exception(&self) -> ArrayOutOfBoundsException {
            ArrayOutOfBoundsException { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> ArrayOutOfBoundsException<'gc_life> {
        as_object_or_java_value!();

        pub fn new<'l, 'k : 'l>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>, index: jint) -> Result<ArrayOutOfBoundsException<'gc_life>, WasException> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, ClassName::Str("java/lang/ArrayOutOfBoundsException".to_string()).into())?;
            push_new_object(jvm, int_state, &class_not_found_class);
            let this = int_state.pop_current_operand_stack(ClassName::object().into());
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

    pub struct IllegalArgumentException<'gc_life> {
        normal_object: Arc<Object<'gc_life>>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_illegal_argument_exception(&self) -> IllegalArgumentException {
            IllegalArgumentException { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> IllegalArgumentException<'gc_life> {
        as_object_or_java_value!();

        pub fn new<'l, 'k : 'l>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>) -> Result<IllegalArgumentException<'gc_life>, WasException> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, ClassName::Str("java/lang/IllegalArgumentException".to_string()).into())?;
            push_new_object(jvm, int_state, &class_not_found_class);
            let this = int_state.pop_current_operand_stack(ClassName::object().into());
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

    pub struct Long<'gc_life> {
        normal_object: Arc<Object<'gc_life>>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_long(&self) -> Long {
            Long { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> Long<'gc_life> {
        as_object_or_java_value!();

        pub fn new<'l, 'k : 'l>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>, param: jlong) -> Result<Long<'gc_life>, WasException> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, ClassName::Str("java/lang/Long".to_string()).into())?;
            push_new_object(jvm, int_state, &class_not_found_class);
            let this = int_state.pop_current_operand_stack(ClassName::long().into());
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

    pub struct Int<'gc_life> {
        normal_object: Arc<Object<'gc_life>>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_int(&self) -> Int {
            Int { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> Int<'gc_life> {
        as_object_or_java_value!();

        pub fn new<'l, 'k : 'l>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>, param: jint) -> Result<Int<'gc_life>, WasException> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, ClassName::Str("java/lang/Int".to_string()).into())?;
            push_new_object(jvm, int_state, &class_not_found_class);
            let this = int_state.pop_current_operand_stack(ClassName::int().into());
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

    pub struct Short<'gc_life> {
        normal_object: Arc<Object<'gc_life>>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_short(&self) -> Short {
            Short { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> Short<'gc_life> {
        as_object_or_java_value!();

        pub fn new<'l, 'k : 'l>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>, param: jshort) -> Result<Short<'gc_life>, WasException> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, ClassName::Str("java/lang/Short".to_string()).into())?;
            push_new_object(jvm, int_state, &class_not_found_class);
            let this = int_state.pop_current_operand_stack(ClassName::short().into());
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

    pub struct Byte<'gc_life> {
        normal_object: Arc<Object<'gc_life>>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_byte(&self) -> Byte {
            Byte { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> Byte<'gc_life> {
        as_object_or_java_value!();

        pub fn new<'l, 'k : 'l>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>, param: jbyte) -> Result<Byte<'gc_life>, WasException> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, ClassName::Str("java/lang/Byte".to_string()).into())?;
            push_new_object(jvm, int_state, &class_not_found_class);
            let this = int_state.pop_current_operand_stack(ClassName::byte().into());
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

    pub struct Boolean<'gc_life> {
        normal_object: Arc<Object<'gc_life>>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_boolean(&self) -> Boolean {
            Boolean { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> Boolean<'gc_life> {
        as_object_or_java_value!();

        pub fn new<'l, 'k : 'l>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>, param: jboolean) -> Result<Boolean<'gc_life>, WasException> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, ClassName::Str("java/lang/Boolean".to_string()).into())?;
            push_new_object(jvm, int_state, &class_not_found_class);
            let this = int_state.pop_current_operand_stack(ClassName::boolean().into());
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

    pub struct Char<'gc_life> {
        normal_object: Arc<Object<'gc_life>>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_char(&self) -> Char {
            Char { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> Char<'gc_life> {
        as_object_or_java_value!();

        pub fn new<'l, 'k : 'l>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>, param: jchar) -> Result<Char<'gc_life>, WasException> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, ClassName::Str("java/lang/Char".to_string()).into())?;
            push_new_object(jvm, int_state, &class_not_found_class);
            let this = int_state.pop_current_operand_stack(ClassName::character().into());
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

    pub struct Float<'gc_life> {
        normal_object: Arc<Object<'gc_life>>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_float(&self) -> Float {
            Float { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> Float<'gc_life> {
        as_object_or_java_value!();

        pub fn new<'l, 'k : 'l>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>, param: jfloat) -> Result<Float<'gc_life>, WasException> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, ClassName::Str("java/lang/Float".to_string()).into())?;
            push_new_object(jvm, int_state, &class_not_found_class);
            let this = int_state.pop_current_operand_stack(ClassName::float().into());
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

    pub struct Double<'gc_life> {
        normal_object: Arc<Object<'gc_life>>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_double(&self) -> Double {
            Double { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> Double<'gc_life> {
        as_object_or_java_value!();

        pub fn new<'l, 'k : 'l>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>, param: jdouble) -> Result<Double<'gc_life>, WasException> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, ClassName::Str("java/lang/Double".to_string()).into())?;
            push_new_object(jvm, int_state, &class_not_found_class);
            let this = int_state.pop_current_operand_stack(ClassName::double().into());
            run_constructor(jvm, int_state, class_not_found_class, vec![this.clone(), JavaValue::Double(param)],
                            "(D)V".to_string())?;
            Ok(this.cast_double())
        }
    }
}


pub mod system;
pub mod reflect;