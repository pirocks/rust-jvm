pub mod invoke;

pub mod throwable {
    use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::names::{CClassName, MethodName};

    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::java_values::{GcManagedObject, JavaValue};
    use crate::jvm_state::JVMState;
    use crate::utils::run_static_or_virtual;

    #[derive(Clone)]
    pub struct Throwable<'gc_life> {
        pub(crate) normal_object: GcManagedObject<'gc_life>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_throwable(&self) -> Throwable<'gc_life> {
            Throwable { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> Throwable<'gc_life> {
        pub fn print_stack_trace(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>) -> Result<(), WasException> {
            let throwable_class = check_initing_or_inited_class(jvm, int_state, CClassName::throwable().into()).expect("Throwable isn't inited?");
            int_state.push_current_operand_stack(JavaValue::Object(self.normal_object.clone().into()));
            run_static_or_virtual(jvm, int_state, &throwable_class, MethodName::method_printStackTrace(), &CMethodDescriptor::empty_args(CPDType::VoidType), todo!())?;
            Ok(())
        }
        as_object_or_java_value!();
    }
}

pub mod stack_trace_element {
    use jvmti_jni_bindings::jint;
    use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::names::CClassName;

    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::interpreter_util::{new_object, run_constructor};
    use crate::java::lang::string::JString;
    use crate::java_values::{GcManagedObject, JavaValue};
    use crate::jvm_state::JVMState;

    #[derive(Clone)]
    pub struct StackTraceElement<'gc_life> {
        normal_object: GcManagedObject<'gc_life>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_stack_trace_element(&self) -> StackTraceElement<'gc_life> {
            StackTraceElement { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> StackTraceElement<'gc_life> {
        pub fn new(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, declaring_class: JString<'gc_life>, method_name: JString<'gc_life>, file_name: JString<'gc_life>, line_number: jint) -> Result<StackTraceElement<'gc_life>, WasException> {
            let class_ = check_initing_or_inited_class(jvm, int_state, CClassName::stack_trace_element().into())?;
            let res = new_object(jvm, int_state, &class_).to_jv();
            let full_args = vec![res.clone(), declaring_class.java_value(), method_name.java_value(), file_name.java_value(), JavaValue::Int(line_number)];
            let desc = CMethodDescriptor::void_return(vec![CClassName::string().into(), CClassName::string().into(), CClassName::string().into(), CPDType::IntType]);
            run_constructor(jvm, int_state, class_, todo!()/*full_args*/, &desc)?;
            Ok(res.cast_stack_trace_element())
        }

        as_object_or_java_value!();
    }
}

pub mod member_name {
    use jvmti_jni_bindings::jint;
    use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName, MethodName};
    use rust_jvm_common::runtime_type::RuntimeType;

    use crate::{InterpreterStateGuard, JVMState};
    use crate::class_loading::{assert_inited_or_initing_class, check_initing_or_inited_class};
    use crate::interpreter::WasException;
    use crate::interpreter_util::{new_object, run_constructor};
    use crate::java::lang::class::JClass;
    use crate::java::lang::invoke::method_type::MethodType;
    use crate::java::lang::reflect::constructor::Constructor;
    use crate::java::lang::reflect::field::Field;
    use crate::java::lang::reflect::method::Method;
    use crate::java::lang::string::JString;
    use crate::java::NewAsObjectOrJavaValue;
    use crate::java_values::{GcManagedObject, JavaValue};
    use crate::utils::run_static_or_virtual;

    #[derive(Clone)]
    pub struct MemberName<'gc_life> {
        normal_object: GcManagedObject<'gc_life>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_member_name(&self) -> MemberName<'gc_life> {
            MemberName { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> MemberName<'gc_life> {
        // private Class<?> clazz;
        // private String name;
        // private Object type;
        // private int flags;
        pub fn get_name_func(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>) -> Result<Option<JString<'gc_life>>, WasException> {
            let member_name_class = assert_inited_or_initing_class(jvm, CClassName::member_name().into());
            int_state.push_current_operand_stack(JavaValue::Object(self.normal_object.clone().into()));
            run_static_or_virtual(jvm, int_state, &member_name_class, MethodName::method_getName(), &CMethodDescriptor::empty_args(CClassName::string().into()), todo!())?;
            Ok(int_state.pop_current_operand_stack(Some(CClassName::string().into())).cast_string())
        }

        pub fn is_static(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>) -> Result<bool, WasException> {
            let member_name_class = assert_inited_or_initing_class(jvm, CClassName::member_name().into());
            int_state.push_current_operand_stack(JavaValue::Object(self.normal_object.clone().into()));
            run_static_or_virtual(jvm, int_state, &member_name_class, MethodName::method_isStatic(), &CMethodDescriptor::empty_args(CPDType::BooleanType), todo!())?;
            Ok(int_state.pop_current_operand_stack(Some(RuntimeType::IntType)).unwrap_boolean() != 0)
        }

        pub fn get_name_or_null(&self, jvm: &'gc_life JVMState<'gc_life>) -> Option<JString<'gc_life>> {
            let str_jvalue = self.normal_object.lookup_field(jvm, FieldName::field_name());
            if str_jvalue.unwrap_object().is_none() {
                None
            } else {
                str_jvalue.cast_string().into()
            }
        }

        pub fn get_name(&self, jvm: &'gc_life JVMState<'gc_life>) -> JString<'gc_life> {
            self.get_name_or_null(jvm).unwrap()
        }

        pub fn set_name(&self, new_val: JString<'gc_life>) {
            self.normal_object.unwrap_normal_object().set_var_top_level(FieldName::field_name(), new_val.java_value());
        }

        pub fn get_clazz_or_null(&self, jvm: &'gc_life JVMState<'gc_life>) -> Option<JClass<'gc_life>> {
            let possibly_null = self.normal_object.unwrap_normal_object().get_var_top_level(jvm, FieldName::field_clazz()).clone();
            if possibly_null.unwrap_object().is_none() {
                None
            } else {
                possibly_null.to_new().cast_class().into()
            }
        }

        pub fn get_clazz(&self, jvm: &'gc_life JVMState<'gc_life>) -> JClass<'gc_life> {
            self.get_clazz_or_null(jvm).unwrap()
        }

        pub fn set_clazz(&self, new_val: JClass<'gc_life>) {
            self.normal_object.unwrap_normal_object().set_var_top_level(FieldName::field_clazz(), new_val.java_value());
        }

        pub fn set_type(&self, new_val: MethodType<'gc_life>) {
            self.normal_object.unwrap_normal_object().set_var_top_level(FieldName::field_type(), new_val.java_value());
        }

        pub fn get_type(&self, jvm: &'gc_life JVMState<'gc_life>) -> JavaValue<'gc_life> {
            self.normal_object.unwrap_normal_object().get_var_top_level(jvm, FieldName::field_type()).clone()
        }

        pub fn set_flags(&self, new_val: jint) {
            self.normal_object.unwrap_normal_object().set_var_top_level(FieldName::field_flags(), JavaValue::Int(new_val));
        }

        pub fn get_flags_or_null(&self, jvm: &'gc_life JVMState<'gc_life>) -> Option<jint> {
            let maybe_null = self.normal_object.lookup_field(jvm, FieldName::field_flags());
            if maybe_null.try_unwrap_object().is_some() {
                if maybe_null.unwrap_object().is_some() {
                    maybe_null.unwrap_int().into()
                } else {
                    None
                }
            } else {
                maybe_null.unwrap_int().into()
            }
        }
        pub fn get_flags(&self, jvm: &'gc_life JVMState<'gc_life>) -> jint {
            self.get_flags_or_null(jvm).unwrap()
        }

        pub fn set_resolution(&self, new_val: JavaValue<'gc_life>) {
            self.normal_object.unwrap_normal_object().set_var_top_level(FieldName::field_resolution(), new_val);
        }

        pub fn get_resolution(&self, jvm: &'gc_life JVMState<'gc_life>) -> JavaValue<'gc_life> {
            self.normal_object.unwrap_normal_object().get_var_top_level(jvm, FieldName::field_resolution()).clone()
        }

        pub fn clazz(&self, jvm: &'gc_life JVMState<'gc_life>) -> Option<JClass<'gc_life>> {
            self.normal_object.unwrap_normal_object().get_var_top_level(jvm, FieldName::field_clazz()).to_new().cast_class()
        }

        pub fn get_method_type(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>) -> Result<MethodType<'gc_life>, WasException> {
            let member_name_class = assert_inited_or_initing_class(jvm, CClassName::member_name().into());
            int_state.push_current_operand_stack(JavaValue::Object(self.normal_object.clone().into()));
            run_static_or_virtual(jvm, int_state, &member_name_class, MethodName::method_getMethodType(), &CMethodDescriptor::empty_args(CClassName::method_type().into()), todo!())?;
            Ok(int_state.pop_current_operand_stack(Some(CClassName::method_type().into())).cast_method_type())
        }

        pub fn get_field_type(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>) -> Result<Option<JClass<'gc_life>>, WasException> {
            let member_name_class = assert_inited_or_initing_class(jvm, CClassName::member_name().into());
            int_state.push_current_operand_stack(JavaValue::Object(self.normal_object.clone().into()));
            run_static_or_virtual(jvm, int_state, &member_name_class, MethodName::method_getFieldType(), &CMethodDescriptor::empty_args(CClassName::class().into()), todo!())?;
            Ok(int_state.pop_current_operand_stack(Some(CClassName::class().into())).to_new().cast_class())
        }

        pub fn new_from_field(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, field: Field<'gc_life>) -> Result<Self, WasException> {
            let member_class = check_initing_or_inited_class(jvm, int_state, CClassName::member_name().into())?;
            let res = new_object(jvm, int_state, &member_class).to_jv();
            run_constructor(jvm, int_state, member_class, todo!()/*vec![res.clone(), field.java_value()]*/, &CMethodDescriptor::void_return(vec![CClassName::field().into()]))?;
            Ok(res.cast_member_name())
        }

        pub fn new_from_method(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, method: Method<'gc_life>) -> Result<Self, WasException> {
            let member_class = check_initing_or_inited_class(jvm, int_state, CClassName::member_name().into())?;
            let res = new_object(jvm, int_state, &member_class).to_jv();
            run_constructor(jvm, int_state, member_class, todo!()/*vec![res.clone(), method.java_value()]*/, &CMethodDescriptor::void_return(vec![CClassName::method().into()]))?;
            Ok(res.cast_member_name())
        }

        pub fn new_from_constructor(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, constructor: Constructor<'gc_life>) -> Result<Self, WasException> {
            let member_class = check_initing_or_inited_class(jvm, int_state, CClassName::member_name().into())?;
            let res = new_object(jvm, int_state, &member_class).to_jv();
            run_constructor(jvm, int_state, member_class, todo!()/*vec![res.clone(), constructor.java_value()]*/, &CMethodDescriptor::void_return(vec![CClassName::constructor().into()]))?;
            Ok(res.cast_member_name())
        }

        as_object_or_java_value!();
    }
}

pub mod class {
    use std::sync::Arc;

    use by_address::ByAddress;

    use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType, CPRefType};
    use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName, MethodName};

    use crate::{InterpreterStateGuard, JVMState, NewJavaValue};
    use crate::class_loading::check_initing_or_inited_class;
    use crate::class_objects::get_or_create_class_object;
    use crate::instructions::ldc::load_class_constant_by_type;
    use crate::interpreter::WasException;
    use crate::interpreter_util::{new_object, run_constructor};
    use crate::java::lang::class_loader::ClassLoader;
    use crate::java::lang::string::JString;
    use crate::java::NewAsObjectOrJavaValue;
    use crate::java_values::{ByAddressAllocatedObject, GcManagedObject, JavaValue};
    use crate::new_java_values::{AllocatedObject, AllocatedObjectCOW, AllocatedObjectHandle, NewJavaValueHandle};
    use crate::runtime_class::RuntimeClass;
    use crate::utils::run_static_or_virtual;

    pub struct JClass<'gc_life> {
        normal_object: AllocatedObjectHandle<'gc_life>,
    }

    impl<'gc_life> Clone for JClass<'gc_life> {
        fn clone(&self) -> Self {
            todo!()
        }
    }

    impl<'gc_life> NewJavaValueHandle<'gc_life> {
        pub fn cast_class(self) -> Option<JClass<'gc_life>> {
            Some(JClass { normal_object: self.unwrap_object()? })
        }
    }

    impl<'gc_life, 'l> NewJavaValue<'gc_life, 'l> {
        pub fn cast_class(&self) -> Option<JClass<'gc_life>> {
            Some(JClass { normal_object: todo!()/*AllocatedObjectCOW::Ref(self.unwrap_object()?.unwrap_alloc())*/ })
        }
    }

    impl<'gc_life> JClass<'gc_life> {
        pub fn as_runtime_class(&self, jvm: &'gc_life JVMState<'gc_life>) -> Arc<RuntimeClass<'gc_life>> {
            jvm.classes.read().unwrap().object_to_runtime_class(self.normal_object.as_allocated_obj())
            //todo I can get rid of this clone since technically only a ref is needed for lookup
        }
        pub fn as_type(&self, jvm: &'gc_life JVMState<'gc_life>) -> CPDType {
            self.as_runtime_class(jvm).cpdtype()
        }
    }

    impl<'gc_life> JClass<'gc_life> {
        pub fn gc_lifeify(&self) -> JClass<'gc_life> {
            todo!()
        }

        pub fn get_class_loader(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>) -> Result<Option<ClassLoader<'gc_life>>, WasException> {
            int_state.push_current_operand_stack(JavaValue::Object(self.normal_object.as_allocated_obj().to_gc_managed().clone().into()));
            run_static_or_virtual(jvm, int_state, &self.normal_object.as_allocated_obj().to_gc_managed().unwrap_normal_object().objinfo.class_pointer, MethodName::method_getClassLoader(), &CMethodDescriptor::empty_args(CClassName::classloader().into()), todo!())?;
            Ok(int_state.pop_current_operand_stack(Some(CClassName::object().into())).unwrap_object().map(|cl| JavaValue::Object(cl.into()).cast_class_loader()))
        }

        pub fn new_bootstrap_loader(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>) -> Result<Self, WasException> {
            let class_class = check_initing_or_inited_class(jvm, int_state, CClassName::class().into())?;
            let res = new_object(jvm, int_state, &class_class);
            run_constructor(jvm, int_state, class_class, vec![res.new_java_value(), NewJavaValue::Null], &CMethodDescriptor::void_return(vec![CClassName::classloader().into()]))?;
            Ok(NewJavaValueHandle::Object(res).cast_class().unwrap())
        }

        pub fn new(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, loader: ClassLoader<'gc_life>) -> Result<Self, WasException> {
            let class_class = check_initing_or_inited_class(jvm, int_state, CClassName::class().into())?;
            let res = new_object(jvm, int_state, &class_class).to_jv();
            run_constructor(jvm, int_state, class_class, todo!()/*vec![res.clone(), loader.java_value()]*/, &CMethodDescriptor::void_return(vec![CClassName::classloader().into()]))?;
            Ok(res.to_new().cast_class().unwrap())
        }

        pub fn from_name(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, name: CClassName) -> JClass<'gc_life> {
            let type_ = CPDType::Ref(CPRefType::Class(name));
            JavaValue::Object(get_or_create_class_object(jvm, type_, int_state).unwrap().to_gc_managed().into()).to_new().cast_class().unwrap()
        }

        pub fn from_type(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, ptype: CPDType) -> Result<JClass<'gc_life>, WasException> {
            let res = load_class_constant_by_type(jvm, int_state, &ptype)?;
            Ok(res.to_handle_discouraged().cast_class().unwrap())//todo we should be able to safely turn handles that live for gc life without reentrant register
        }

        pub fn get_name(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>) -> Result<JString<'gc_life>, WasException> {
            int_state.push_current_operand_stack(self.clone().java_value());
            let class_class = check_initing_or_inited_class(jvm, int_state, CClassName::class().into()).unwrap();
            run_static_or_virtual(jvm, int_state, &class_class, MethodName::method_getName(), &CMethodDescriptor::empty_args(CClassName::string().into()), todo!())?;
            let result_popped_from_operand_stack: JavaValue<'gc_life> = int_state.pop_current_operand_stack(Some(CClassName::string().into()));
            Ok(result_popped_from_operand_stack.cast_string().expect("classes are known to have non-null names"))
        }

        pub fn set_name_(&self, name: JString<'gc_life>) {
            let temp = self.normal_object.as_allocated_obj().to_gc_managed();
            let normal_object = temp.unwrap_normal_object();
            normal_object.set_var_top_level(FieldName::field_name(), name.java_value());
        }

        pub fn object_gc_life(self, jvm: &JVMState<'gc_life>) -> AllocatedObject<'gc_life, 'gc_life> {
            jvm.gc.handle_lives_for_gc_life(self.normal_object)
        }
    }

    impl<'gc_life> NewAsObjectOrJavaValue<'gc_life> for JClass<'gc_life> {
        fn object(self) -> AllocatedObjectHandle<'gc_life> {
            self.normal_object
        }
    }
}

pub mod class_loader {
    use by_address::ByAddress;

    use rust_jvm_common::compressed_classfile::CMethodDescriptor;
    use rust_jvm_common::compressed_classfile::names::{CClassName, MethodName};
    use rust_jvm_common::loading::LoaderName;

    use crate::class_loading::assert_inited_or_initing_class;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::java::lang::class::JClass;
    use crate::java::lang::string::JString;
    use crate::java_values::{GcManagedObject, JavaValue};
    use crate::jvm_state::JVMState;
    use crate::utils::run_static_or_virtual;

    #[derive(Clone)]
    pub struct ClassLoader<'gc_life> {
        normal_object: GcManagedObject<'gc_life>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_class_loader(&self) -> ClassLoader<'gc_life> {
            ClassLoader { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> ClassLoader<'gc_life> {
        pub fn to_jvm_loader(&self, jvm: &'gc_life JVMState<'gc_life>) -> LoaderName {
            let mut classes_guard = jvm.classes.write().unwrap();
            classes_guard.lookup_or_add_classloader(todo!()/*self.normal_object.clone()*/)
        }

        pub fn load_class(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, name: JString<'gc_life>) -> Result<JClass<'gc_life>, WasException> {
            int_state.push_current_operand_stack(self.clone().java_value());
            int_state.push_current_operand_stack(name.java_value());
            let class_loader = assert_inited_or_initing_class(jvm, CClassName::classloader().into());
            run_static_or_virtual(jvm, int_state, &class_loader, MethodName::method_loadClass(), &CMethodDescriptor { arg_types: vec![CClassName::string().into()], return_type: CClassName::class().into() }, todo!())?;
            assert!(int_state.throw().is_none());
            Ok(int_state.pop_current_operand_stack(Some(CClassName::class().into())).to_new().cast_class().unwrap())
        }

        pub fn object(self) -> crate::new_java_values::AllocatedObject<'gc_life, 'gc_life> {
            todo!()
        }

        as_object_or_java_value!();
    }
}

pub mod string {
    use std::cell::UnsafeCell;

    use itertools::Itertools;
    use libc::c_void;
    use wtf8::Wtf8Buf;

    use jvmti_jni_bindings::{jchar, jint};
    use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName, MethodName};

    use crate::{InterpreterStateGuard, JVMState, NewJavaValue};
    use crate::class_loading::{assert_inited_or_initing_class, check_initing_or_inited_class};
    use crate::interpreter::WasException;
    use crate::interpreter_util::{new_object, run_constructor};
    use crate::java::NewAsObjectOrJavaValue;
    use crate::java_values::{ArrayObject, GcManagedObject, JavaValue, Object};
    use crate::new_java_values::{AllocatedObjectHandle, NewJavaValueHandle, UnAllocatedObject, UnAllocatedObjectArray};
    use crate::utils::run_static_or_virtual;
    use crate::utils::string_obj_to_string;

    pub struct JString<'gc_life> {
        normal_object: AllocatedObjectHandle<'gc_life>,
    }

    impl Clone for JString<'_> {
        fn clone(&self) -> Self {
            todo!()
        }
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_string(&self) -> Option<JString<'gc_life>> {
            todo!()
            /*Some(JString { normal_object: self.unwrap_object()? })*/
        }
    }

    impl<'gc_life> NewJavaValueHandle<'gc_life> {
        pub fn cast_string(self) -> Option<JString<'gc_life>> {
            Some(JString { normal_object: self.unwrap_object()? })
        }
    }

    impl<'gc_life> JString<'gc_life> {
        pub fn to_rust_string(&self, jvm: &'gc_life JVMState<'gc_life>) -> String {
            string_obj_to_string(jvm, self.normal_object.as_allocated_obj())
        }

        pub fn from_rust(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, rust_str: Wtf8Buf) -> Result<JString<'gc_life>, WasException> {
            let string_class = check_initing_or_inited_class(jvm, int_state, CClassName::string().into()).unwrap(); //todo replace these unwraps
            let string_object = new_object(jvm, int_state, &string_class);

            let elems = rust_str.to_ill_formed_utf16().map(|c| NewJavaValue::Char(c as u16)).collect_vec();
            let array_object = UnAllocatedObjectArray {
                whole_array_runtime_class: check_initing_or_inited_class(jvm, int_state, CPDType::array(CPDType::CharType)).unwrap(),
                elems,
            };
            //todo what about check_inited_class for this array type
            let array = NewJavaValueHandle::Object(jvm.allocate_object(UnAllocatedObject::Array(array_object)));
            run_constructor(jvm, int_state, string_class, vec![string_object.new_java_value(), array.as_njv()], &CMethodDescriptor::void_return(vec![CPDType::array(CPDType::CharType)]))?;
            Ok(NewJavaValueHandle::Object(string_object).cast_string().expect("error creating string"))
        }

        pub fn intern(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>) -> Result<JString<'gc_life>, WasException> {
            int_state.push_current_operand_stack(self.clone().java_value());
            let string_class = check_initing_or_inited_class(jvm, int_state, CClassName::string().into())?;
            run_static_or_virtual(jvm, int_state, &string_class, MethodName::method_intern(), &CMethodDescriptor::empty_args(CClassName::string().into()), todo!())?;
            Ok(int_state.pop_current_operand_stack(Some(CClassName::string().into())).cast_string().expect("error interning strinng"))
        }

        pub fn value(&self, jvm: &'gc_life JVMState<'gc_life>) -> Vec<jchar> {
            let string_class = assert_inited_or_initing_class(jvm, CClassName::string().into());
            let mut res = vec![];
            for elem in self.normal_object.as_allocated_obj().lookup_field(&string_class, FieldName::field_value()).unwrap_array(jvm).array_iterator() {
                res.push(elem.as_njv().unwrap_char_strict())
            }
            res
        }

        pub fn length(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>) -> Result<jint, WasException> {
            int_state.push_current_operand_stack(self.clone().java_value());
            let string_class = check_initing_or_inited_class(jvm, int_state, CClassName::string().into())?;
            run_static_or_virtual(jvm, int_state, &string_class, MethodName::method_length(), &CMethodDescriptor::empty_args(CPDType::IntType), todo!())?;
            Ok(int_state.pop_current_operand_stack(Some(CClassName::string().into())).unwrap_int())
        }

        as_object_or_java_value!();
    }

    impl <'gc_life> NewAsObjectOrJavaValue<'gc_life> for JString<'gc_life>{
        fn object(self) -> AllocatedObjectHandle<'gc_life> {
            self.normal_object
        }
    }
}

pub mod integer {
    use jvmti_jni_bindings::jint;
    use rust_jvm_common::compressed_classfile::names::FieldName;

    use crate::{JVMState, StackEntry};
    use crate::java_values::{GcManagedObject, JavaValue};

    pub struct Integer<'gc_life> {
        normal_object: GcManagedObject<'gc_life>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_integer(&self) -> Integer<'gc_life> {
            Integer { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> Integer<'gc_life> {
        pub fn from(_state: &JVMState, _current_frame: &StackEntry, _i: jint) -> Integer<'gc_life> {
            unimplemented!()
        }

        pub fn value(&self, jvm: &'gc_life JVMState<'gc_life>) -> jint {
            self.normal_object.unwrap_normal_object().get_var_top_level(jvm, FieldName::field_value()).unwrap_int()
        }

        as_object_or_java_value!();
    }
}

pub mod object {
    use crate::java_values::{GcManagedObject, JavaValue};

    pub struct JObject<'gc_life> {
        normal_object: GcManagedObject<'gc_life>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_object(&self) -> JObject<'gc_life> {
            JObject { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> JObject<'gc_life> {
        as_object_or_java_value!();
    }
}

pub mod thread {
    use std::cell::UnsafeCell;
    use std::ptr::null_mut;
    use std::sync::Arc;

    use itertools::Itertools;
    use wtf8::Wtf8Buf;

    use jvmti_jni_bindings::{jboolean, jint};
    use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CompressedMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName, MethodName};
    use rust_jvm_common::JavaThreadId;
    use rust_jvm_common::runtime_type::RuntimeType;

    use crate::{InterpreterStateGuard, JVMState, NewJavaValue};
    use crate::class_loading::assert_inited_or_initing_class;
    use crate::interpreter::WasException;
    use crate::interpreter_util::{new_object, run_constructor};
    use crate::java::lang::class_loader::ClassLoader;
    use crate::java::lang::string::JString;
    use crate::java::lang::thread_group::JThreadGroup;
    use crate::java_values::{GcManagedObject, NativeJavaValue, NormalObject, Object, ObjectFieldsAndClass};
    use crate::java_values::JavaValue;
    use crate::runtime_class::RuntimeClass;
    use crate::threading::JavaThread;
    use crate::utils::run_static_or_virtual;

    #[derive(Clone)]
    pub struct JThread<'gc_life> {
        normal_object: GcManagedObject<'gc_life>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_thread(&self) -> JThread<'gc_life> {
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
                None => None,
            }
        }
    }

    impl<'gc_life> JThread<'gc_life> {
        pub fn invalid_thread(jvm: &'gc_life JVMState<'gc_life>) -> JThread<'gc_life> {
            const NUMBER_OF_LOCAL_VARS_IN_THREAD: i32 = 16;
            JThread {
                normal_object: NewJavaValue::AllocObject(todo!()/*jvm.allocate_object(todo!()/*Object::Object(NormalObject {
                    /*monitor: jvm.thread_state.new_monitor("invalid thread monitor".to_string()),

                    objinfo: ObjectFieldsAndClass {
                        fields: (0..NUMBER_OF_LOCAL_VARS_IN_THREAD).map(|_| UnsafeCell::new(NativeJavaValue { object: null_mut() })).collect_vec(),
                        class_pointer: Arc::new(RuntimeClass::Top),
                    },*/
                    objinfo: todo!(),
                    obj_ptr: todo!(),
                })*/)*/).to_jv().unwrap_object_nonnull(),
            }
        }

        pub fn tid(&self, jvm: &'gc_life JVMState<'gc_life>) -> JavaThreadId {
            let thread_class = assert_inited_or_initing_class(jvm, CClassName::thread().into());
            self.normal_object.unwrap_normal_object().get_var(jvm, thread_class, FieldName::field_tid()).unwrap_long()
        }

        pub fn run(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>) -> Result<(), WasException> {
            let thread_class = self.normal_object.unwrap_normal_object().objinfo.class_pointer.clone();
            int_state.push_current_operand_stack(JavaValue::Object(self.normal_object.clone().into()));
            run_static_or_virtual(jvm, int_state, &thread_class, MethodName::method_run(), &CompressedMethodDescriptor::empty_args(CPDType::VoidType), todo!())
        }

        pub fn exit(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>) -> Result<(), WasException> {
            let thread_class = self.normal_object.unwrap_normal_object().objinfo.class_pointer.clone();
            int_state.push_current_operand_stack(JavaValue::Object(self.normal_object.clone().into()));
            run_static_or_virtual(jvm, int_state, &thread_class, MethodName::method_exit(), &CompressedMethodDescriptor::empty_args(CPDType::VoidType), todo!())
        }

        pub fn name(&self, jvm: &'gc_life JVMState<'gc_life>) -> JString<'gc_life> {
            let thread_class = assert_inited_or_initing_class(jvm, CClassName::thread().into());
            self.normal_object.unwrap_normal_object().get_var(jvm, thread_class, FieldName::field_name()).cast_string().expect("threads are known to have nonnull names")
        }

        pub fn priority(&self, jvm: &'gc_life JVMState<'gc_life>) -> i32 {
            let thread_class = assert_inited_or_initing_class(jvm, CClassName::thread().into());
            self.normal_object.unwrap_normal_object().get_var(jvm, thread_class, FieldName::field_priority()).unwrap_int()
        }

        pub fn set_priority(&self, priority: i32) {
            self.normal_object.unwrap_normal_object().set_var_top_level(FieldName::field_priority(), JavaValue::Int(priority));
        }

        pub fn daemon(&self, jvm: &'gc_life JVMState<'gc_life>) -> bool {
            let thread_class = assert_inited_or_initing_class(jvm, CClassName::thread().into());
            self.normal_object.unwrap_normal_object().get_var(jvm, thread_class, FieldName::field_daemon()).unwrap_int() != 0
        }

        pub fn set_thread_status(&self, jvm: &'gc_life JVMState<'gc_life>, thread_status: jint) {
            let thread_class = assert_inited_or_initing_class(jvm, CClassName::thread().into());
            self.normal_object.unwrap_normal_object().set_var(thread_class, FieldName::field_threadStatus(), JavaValue::Int(thread_status));
        }

        pub fn new(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, thread_group: JThreadGroup<'gc_life>, thread_name: String) -> Result<JThread<'gc_life>, WasException> {
            let thread_class = assert_inited_or_initing_class(jvm, CClassName::thread().into());
            let thread_object = new_object(jvm, int_state, &thread_class).to_jv();
            let thread_name = JString::from_rust(jvm, int_state, Wtf8Buf::from_string(thread_name))?;
            run_constructor(jvm, int_state, thread_class, todo!()/*vec![thread_object.clone(), thread_group.java_value(), thread_name.java_value()]*/, &CMethodDescriptor::void_return(vec![CClassName::thread_group().into(), CClassName::string().into()]))?;
            Ok(thread_object.cast_thread())
        }

        pub fn get_java_thread(&self, jvm: &'gc_life JVMState<'gc_life>) -> Arc<JavaThread<'gc_life>> {
            self.try_get_java_thread(jvm).unwrap()
        }

        pub fn try_get_java_thread(&self, jvm: &'gc_life JVMState<'gc_life>) -> Option<Arc<JavaThread<'gc_life>>> {
            let tid = self.tid(jvm);
            jvm.thread_state.try_get_thread_by_tid(tid)
        }

        pub fn is_alive(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>) -> Result<jboolean, WasException> {
            let thread_class = assert_inited_or_initing_class(jvm, CClassName::thread().into());
            int_state.push_current_operand_stack(self.clone().java_value());
            run_static_or_virtual(jvm, int_state, &thread_class, MethodName::method_isAlive(), &CompressedMethodDescriptor::empty_args(CPDType::BooleanType), todo!())?;
            Ok(int_state.pop_current_operand_stack(Some(RuntimeType::IntType)).unwrap_boolean())
        }

        pub fn get_context_class_loader(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>) -> Result<Option<ClassLoader<'gc_life>>, WasException> {
            let thread_class = assert_inited_or_initing_class(jvm, CClassName::thread().into());
            int_state.push_current_operand_stack(self.clone().java_value());
            run_static_or_virtual(jvm, int_state, &thread_class, MethodName::method_getContextClassLoader(), &CompressedMethodDescriptor::empty_args(CClassName::classloader().into()), todo!())?;
            let res = int_state.pop_current_operand_stack(Some(CClassName::classloader().into()));
            if res.unwrap_object().is_none() {
                return Ok(None);
            }
            Ok(res.cast_class_loader().into())
        }

        pub fn get_inherited_access_control_context(&self, jvm: &'gc_life JVMState<'gc_life>) -> JThread<'gc_life> {
            self.normal_object.lookup_field(jvm, FieldName::field_inheritedAccessControlContext()).cast_thread()
        }

        pub fn object(self) -> crate::new_java_values::AllocatedObject<'gc_life, 'gc_life> {
            todo!()
        }

        as_object_or_java_value!();
    }
}

pub mod thread_group {
    use std::sync::Arc;

    use jvmti_jni_bindings::{jboolean, jint};
    use rust_jvm_common::compressed_classfile::CMethodDescriptor;
    use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName};

    use crate::{InterpreterStateGuard, JVMState};
    use crate::interpreter::WasException;
    use crate::interpreter_util::{new_object, run_constructor};
    use crate::java::lang::string::JString;
    use crate::java::lang::thread::JThread;
    use crate::java_values::{GcManagedObject, JavaValue};
    use crate::runtime_class::RuntimeClass;

    #[derive(Clone)]
    pub struct JThreadGroup<'gc_life> {
        normal_object: GcManagedObject<'gc_life>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_thread_group(&self) -> JThreadGroup<'gc_life> {
            JThreadGroup { normal_object: self.unwrap_object_nonnull() }
        }

        pub fn try_cast_thread_group(&self) -> Option<JThreadGroup<'gc_life>> {
            match self.try_unwrap_normal_object() {
                Some(normal_object) => {
                    if normal_object.objinfo.class_pointer.view().name() == CClassName::thread_group().into() {
                        return JThreadGroup { normal_object: self.unwrap_object_nonnull() }.into();
                    }
                    None
                }
                None => None,
            }
        }
    }

    impl<'gc_life> JThreadGroup<'gc_life> {
        pub fn init(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, thread_group_class: Arc<RuntimeClass<'gc_life>>) -> Result<JThreadGroup<'gc_life>, WasException> {
            let thread_group_object = new_object(jvm, int_state, &thread_group_class).to_jv();
            run_constructor(jvm, int_state, thread_group_class, todo!()/*vec![thread_group_object.clone()]*/, &CMethodDescriptor::void_return(vec![]))?;
            Ok(thread_group_object.cast_thread_group())
        }

        pub fn threads(&self, jvm: &'gc_life JVMState<'gc_life>) -> Vec<Option<JThread<'gc_life>>> {
            let threads_field = self.normal_object.lookup_field(jvm, FieldName::field_threads());
            let array = threads_field.unwrap_array();
            array
                .array_iterator(jvm)
                .map(|thread| match thread.unwrap_object() {
                    None => None,
                    Some(t) => JavaValue::Object(t.into()).cast_thread().into(),
                })
                .collect()
        }

        pub fn threads_non_null(&self, jvm: &'gc_life JVMState<'gc_life>) -> Vec<JThread<'gc_life>> {
            self.threads(jvm).into_iter().flatten().collect()
        }

        pub fn name(&self, jvm: &'gc_life JVMState<'gc_life>) -> JString<'gc_life> {
            self.normal_object.lookup_field(jvm, FieldName::field_name()).cast_string().expect("thread group null name")
        }

        pub fn daemon(&self, jvm: &'gc_life JVMState<'gc_life>) -> jboolean {
            self.normal_object.lookup_field(jvm, FieldName::field_daemon()).unwrap_boolean()
        }

        pub fn max_priority(&self, jvm: &'gc_life JVMState<'gc_life>) -> jint {
            self.normal_object.lookup_field(jvm, FieldName::field_maxPriority()).unwrap_int()
        }

        pub fn parent(&self, jvm: &'gc_life JVMState<'gc_life>) -> Option<JThreadGroup<'gc_life>> {
            self.normal_object.lookup_field(jvm, FieldName::field_parent()).try_cast_thread_group()
        }

        pub fn object(self) -> crate::new_java_values::AllocatedObject<'gc_life, 'gc_life> {
            todo!()
        }
        as_object_or_java_value!();
    }
}

pub mod class_not_found_exception {
    use rust_jvm_common::compressed_classfile::CMethodDescriptor;
    use rust_jvm_common::compressed_classfile::names::CClassName;

    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::interpreter_util::{new_object, run_constructor};
    use crate::java::lang::string::JString;
    use crate::java_values::{GcManagedObject, JavaValue};
    use crate::jvm_state::JVMState;

    pub struct ClassNotFoundException<'gc_life> {
        normal_object: GcManagedObject<'gc_life>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_class_not_found_exception(&self) -> ClassNotFoundException<'gc_life> {
            ClassNotFoundException { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> ClassNotFoundException<'gc_life> {
        as_object_or_java_value!();

        pub fn new(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, class: JString<'gc_life>) -> Result<ClassNotFoundException<'gc_life>, WasException> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, CClassName::class_not_found_exception().into())?;
            let this = new_object(jvm, int_state, &class_not_found_class).to_jv();
            run_constructor(jvm, int_state, class_not_found_class, todo!()/*vec![this.clone(), class.java_value()]*/, &CMethodDescriptor::void_return(vec![CClassName::string().into()]))?;
            Ok(this.cast_class_not_found_exception())
        }
    }
}

pub mod null_pointer_exception {
    use wtf8::Wtf8Buf;

    use rust_jvm_common::compressed_classfile::CMethodDescriptor;
    use rust_jvm_common::compressed_classfile::names::CClassName;

    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::interpreter_util::{new_object, run_constructor};
    use crate::java::lang::string::JString;
    use crate::java_values::{GcManagedObject, JavaValue};
    use crate::jvm_state::JVMState;

    pub struct NullPointerException<'gc_life> {
        normal_object: GcManagedObject<'gc_life>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_null_pointer_exception(&self) -> NullPointerException<'gc_life> {
            NullPointerException { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> NullPointerException<'gc_life> {
        as_object_or_java_value!();

        pub fn new(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>) -> Result<NullPointerException<'gc_life>, WasException> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, CClassName::null_pointer_exception().into())?;
            let this = new_object(jvm, int_state, &class_not_found_class).to_jv();
            let message = JString::from_rust(jvm, int_state, Wtf8Buf::from_string("This jvm doesn't believe in helpful null pointer messages so you get this instead".to_string()))?;
            run_constructor(jvm, int_state, class_not_found_class, todo!()/*vec![this.clone(), message.java_value()]*/, &CMethodDescriptor::void_return(vec![CClassName::string().into()]))?;
            Ok(this.cast_null_pointer_exception())
        }
    }
}

pub mod array_out_of_bounds_exception {
    use jvmti_jni_bindings::jint;
    use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::names::CClassName;

    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::interpreter_util::{new_object, run_constructor};
    use crate::java_values::{GcManagedObject, JavaValue};
    use crate::jvm_state::JVMState;

    pub struct ArrayOutOfBoundsException<'gc_life> {
        normal_object: GcManagedObject<'gc_life>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_array_out_of_bounds_exception(&self) -> ArrayOutOfBoundsException<'gc_life> {
            ArrayOutOfBoundsException { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> ArrayOutOfBoundsException<'gc_life> {
        as_object_or_java_value!();

        pub fn new(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, index: jint) -> Result<ArrayOutOfBoundsException<'gc_life>, WasException> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, CClassName::array_out_of_bounds_exception().into())?;
            let this = new_object(jvm, int_state, &class_not_found_class).to_jv();
            run_constructor(jvm, int_state, class_not_found_class, todo!()/*vec![this.clone(), JavaValue::Int(index)]*/, &CMethodDescriptor::void_return(vec![CPDType::IntType]))?;
            Ok(this.cast_array_out_of_bounds_exception())
        }
    }
}

pub mod illegal_argument_exception {
    use rust_jvm_common::compressed_classfile::CMethodDescriptor;
    use rust_jvm_common::compressed_classfile::names::CClassName;

    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::interpreter_util::{new_object, run_constructor};
    use crate::java_values::{GcManagedObject, JavaValue};
    use crate::jvm_state::JVMState;

    pub struct IllegalArgumentException<'gc_life> {
        normal_object: GcManagedObject<'gc_life>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_illegal_argument_exception(&self) -> IllegalArgumentException<'gc_life> {
            IllegalArgumentException { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> IllegalArgumentException<'gc_life> {
        as_object_or_java_value!();

        pub fn new(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>) -> Result<IllegalArgumentException<'gc_life>, WasException> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, CClassName::illegal_argument_exception().into())?;
            let this = new_object(jvm, int_state, &class_not_found_class).to_jv();
            run_constructor(jvm, int_state, class_not_found_class, todo!()/*vec![this.clone()]*/, &CMethodDescriptor::void_return(vec![]))?;
            Ok(this.cast_illegal_argument_exception())
        }

        pub fn object(self) -> crate::new_java_values::AllocatedObject<'gc_life, 'todo> {
            /*self.normal_object*/
            todo!()
        }
    }
}

pub mod long {
    use jvmti_jni_bindings::jlong;
    use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName};

    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::interpreter_util::{new_object, run_constructor};
    use crate::java_values::{GcManagedObject, JavaValue};
    use crate::jvm_state::JVMState;

    pub struct Long<'gc_life> {
        normal_object: GcManagedObject<'gc_life>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_long(&self) -> Long<'gc_life> {
            Long { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> Long<'gc_life> {
        as_object_or_java_value!();

        pub fn new(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, param: jlong) -> Result<Long<'gc_life>, WasException> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, CClassName::long().into())?;
            let this = new_object(jvm, int_state, &class_not_found_class).to_jv();
            run_constructor(jvm, int_state, class_not_found_class, todo!()/*vec![this.clone(), JavaValue::Long(param)]*/, &CMethodDescriptor::void_return(vec![CPDType::LongType]))?;
            Ok(this.cast_long())
        }

        pub fn inner_value(&self, jvm: &'gc_life JVMState<'gc_life>) -> jlong {
            self.normal_object.lookup_field(jvm, FieldName::field_value()).unwrap_long()
        }
        pub fn object(self) -> crate::new_java_values::AllocatedObject<'gc_life, 'todo> {
            /*self.normal_object*/
            todo!()
        }
    }
}

pub mod int {
    use jvmti_jni_bindings::jint;
    use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName};

    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::interpreter_util::{new_object, run_constructor};
    use crate::java_values::{GcManagedObject, JavaValue};
    use crate::jvm_state::JVMState;

    pub struct Int<'gc_life> {
        normal_object: GcManagedObject<'gc_life>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_int(&self) -> Int<'gc_life> {
            Int { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> Int<'gc_life> {
        as_object_or_java_value!();

        pub fn new(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, param: jint) -> Result<Int<'gc_life>, WasException> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, CClassName::int().into())?;
            let this = new_object(jvm, int_state, &class_not_found_class).to_jv();
            run_constructor(jvm, int_state, class_not_found_class, todo!()/*vec![this.clone(), JavaValue::Int(param)]*/, &CMethodDescriptor::void_return(vec![CPDType::IntType]))?;
            Ok(this.cast_int())
        }

        pub fn inner_value(&self, jvm: &'gc_life JVMState<'gc_life>) -> jint {
            self.normal_object.lookup_field(jvm, FieldName::field_value()).unwrap_int()
        }
        pub fn object(self) -> crate::new_java_values::AllocatedObject<'gc_life, 'todo> {
            /*self.normal_object*/
            todo!()
        }
    }
}

pub mod short {
    use jvmti_jni_bindings::jshort;
    use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName};

    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::interpreter_util::{new_object, run_constructor};
    use crate::java_values::{GcManagedObject, JavaValue};
    use crate::jvm_state::JVMState;

    pub struct Short<'gc_life> {
        normal_object: GcManagedObject<'gc_life>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_short(&self) -> Short<'gc_life> {
            Short { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> Short<'gc_life> {
        as_object_or_java_value!();

        pub fn new(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, param: jshort) -> Result<Short<'gc_life>, WasException> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, CClassName::short().into())?;
            let this = new_object(jvm, int_state, &class_not_found_class).to_jv();
            run_constructor(jvm, int_state, class_not_found_class, todo!()/*vec![this.clone(), JavaValue::Short(param)]*/, &CMethodDescriptor::void_return(vec![CPDType::ShortType]))?;
            Ok(this.cast_short())
        }

        pub fn inner_value(&self, jvm: &'gc_life JVMState<'gc_life>) -> jshort {
            self.normal_object.lookup_field(jvm, FieldName::field_value()).unwrap_short()
        }
        pub fn object(self) -> crate::new_java_values::AllocatedObject<'gc_life, 'todo> {
            /*self.normal_object*/
            todo!()
        }
    }
}

pub mod byte {
    use jvmti_jni_bindings::jbyte;
    use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName};

    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::interpreter_util::{new_object, run_constructor};
    use crate::java_values::{GcManagedObject, JavaValue};
    use crate::jvm_state::JVMState;

    pub struct Byte<'gc_life> {
        normal_object: GcManagedObject<'gc_life>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_byte(&self) -> Byte<'gc_life> {
            Byte { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> Byte<'gc_life> {
        as_object_or_java_value!();

        pub fn new(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, param: jbyte) -> Result<Byte<'gc_life>, WasException> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, CClassName::byte().into())?;
            let this = new_object(jvm, int_state, &class_not_found_class).to_jv();
            run_constructor(jvm, int_state, class_not_found_class, todo!()/*vec![this.clone(), JavaValue::Byte(param)]*/, &CMethodDescriptor::void_return(vec![CPDType::ByteType]))?;
            Ok(this.cast_byte())
        }

        pub fn inner_value(&self, jvm: &'gc_life JVMState<'gc_life>) -> jbyte {
            self.normal_object.lookup_field(jvm, FieldName::field_value()).unwrap_byte()
        }
        pub fn object(self) -> crate::new_java_values::AllocatedObject<'gc_life, 'todo> {
            /*self.normal_object*/
            todo!()
        }
    }
}

pub mod boolean {
    use jvmti_jni_bindings::jboolean;
    use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName};

    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::interpreter_util::{new_object, run_constructor};
    use crate::java_values::{GcManagedObject, JavaValue};
    use crate::jvm_state::JVMState;

    pub struct Boolean<'gc_life> {
        normal_object: GcManagedObject<'gc_life>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_boolean(&self) -> Boolean<'gc_life> {
            Boolean { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> Boolean<'gc_life> {
        as_object_or_java_value!();

        pub fn new(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, param: jboolean) -> Result<Boolean<'gc_life>, WasException> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, CClassName::boolean().into())?;
            let this = new_object(jvm, int_state, &class_not_found_class).to_jv();
            run_constructor(jvm, int_state, class_not_found_class, todo!()/*vec![this.clone(), JavaValue::Boolean(param)]*/, &CMethodDescriptor::void_return(vec![CPDType::BooleanType]))?;
            Ok(this.cast_boolean())
        }

        pub fn inner_value(&self, jvm: &'gc_life JVMState<'gc_life>) -> jboolean {
            self.normal_object.lookup_field(jvm, FieldName::field_value()).unwrap_boolean()
        }
        pub fn object(self) -> crate::new_java_values::AllocatedObject<'gc_life, 'todo> {
            /*self.normal_object*/
            todo!()
        }
    }
}

pub mod char {
    use jvmti_jni_bindings::jchar;
    use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName};

    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::interpreter_util::{new_object, run_constructor};
    use crate::java_values::{GcManagedObject, JavaValue};
    use crate::jvm_state::JVMState;

    pub struct Char<'gc_life> {
        normal_object: GcManagedObject<'gc_life>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_char(&self) -> Char<'gc_life> {
            Char { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> Char<'gc_life> {
        as_object_or_java_value!();

        pub fn new(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, param: jchar) -> Result<Char<'gc_life>, WasException> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, CClassName::character().into())?;
            let this = new_object(jvm, int_state, &class_not_found_class).to_jv();
            run_constructor(jvm, int_state, class_not_found_class, todo!()/*vec![this.clone(), JavaValue::Char(param)]*/, &CMethodDescriptor::void_return(vec![CPDType::CharType]))?;
            Ok(this.cast_char())
        }

        pub fn inner_value(&self, jvm: &'gc_life JVMState<'gc_life>) -> jchar {
            self.normal_object.lookup_field(jvm, FieldName::field_value()).unwrap_char()
        }
        pub fn object(self) -> crate::new_java_values::AllocatedObject<'gc_life, 'todo> {
            /*self.normal_object*/
            todo!()
        }
    }
}

pub mod float {
    use jvmti_jni_bindings::jfloat;
    use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName};

    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::interpreter_util::{new_object, run_constructor};
    use crate::java_values::{GcManagedObject, JavaValue};
    use crate::jvm_state::JVMState;

    pub struct Float<'gc_life> {
        normal_object: GcManagedObject<'gc_life>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_float(&self) -> Float<'gc_life> {
            Float { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> Float<'gc_life> {
        as_object_or_java_value!();

        pub fn new(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, param: jfloat) -> Result<Float<'gc_life>, WasException> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, CClassName::float().into())?;
            let this = new_object(jvm, int_state, &class_not_found_class).to_jv();
            run_constructor(jvm, int_state, class_not_found_class, todo!()/*vec![this.clone(), JavaValue::Float(param)]*/, &CMethodDescriptor::void_return(vec![CPDType::FloatType]))?;
            Ok(this.cast_float())
        }

        pub fn inner_value(&self, jvm: &'gc_life JVMState<'gc_life>) -> jfloat {
            self.normal_object.lookup_field(jvm, FieldName::field_value()).unwrap_float()
        }
        pub fn object(self) -> crate::new_java_values::AllocatedObject<'gc_life, 'todo> {
            /*self.normal_object*/
            todo!()
        }
    }
}

pub mod double {
    use jvmti_jni_bindings::jdouble;
    use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName};

    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::interpreter_util::{new_object, run_constructor};
    use crate::java_values::{GcManagedObject, JavaValue};
    use crate::jvm_state::JVMState;

    pub struct Double<'gc_life> {
        normal_object: GcManagedObject<'gc_life>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_double(&self) -> Double<'gc_life> {
            Double { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> Double<'gc_life> {
        as_object_or_java_value!();

        pub fn new(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, param: jdouble) -> Result<Double<'gc_life>, WasException> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, CClassName::double().into())?;
            let this = new_object(jvm, int_state, &class_not_found_class).to_jv();
            run_constructor(jvm, int_state, class_not_found_class, todo!()/*vec![this.clone(), JavaValue::Double(param)]*/, &CMethodDescriptor::void_return(vec![CPDType::DoubleType]))?;
            Ok(this.cast_double())
        }

        pub fn inner_value(&self, jvm: &'gc_life JVMState<'gc_life>) -> jdouble {
            self.normal_object.lookup_field(jvm, FieldName::field_value()).unwrap_double()
        }
        pub fn object(self) -> crate::new_java_values::AllocatedObject<'gc_life, 'todo> {
            /*self.normal_object*/
            todo!()
        }
    }
}

pub mod reflect;
pub mod system;