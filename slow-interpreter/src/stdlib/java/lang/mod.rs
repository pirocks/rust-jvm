pub mod invoke;

pub mod throwable {
    use rust_jvm_common::compressed_classfile::class_names::CClassName;
    use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::method_names::MethodName;
    use crate::{NewAsObjectOrJavaValue, WasException};
    use crate::better_java_stack::frames::PushableFrame;
    use crate::class_loading::check_initing_or_inited_class;
    use crate::jvm_state::JVMState;
    use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
    use crate::utils::run_static_or_virtual;

    pub struct Throwable<'gc> {
        pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
    }

    impl<'gc> Clone for Throwable<'gc> {
        fn clone(&self) -> Self {
            Throwable { normal_object: self.normal_object.duplicate_discouraged() }
        }
    }

    impl<'gc> Throwable<'gc> {
        pub fn print_stack_trace<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<(), WasException<'gc>> {
            let throwable_class = check_initing_or_inited_class(jvm, int_state, CClassName::throwable().into()).expect("Throwable isn't inited?");
            let args = vec![self.new_java_value()];
            run_static_or_virtual(jvm, int_state, &throwable_class, MethodName::method_printStackTrace(), &CMethodDescriptor::empty_args(CPDType::VoidType), args)?;
            Ok(())
        }
    }

    impl<'gc> NewAsObjectOrJavaValue<'gc> for Throwable<'gc> {
        fn object(self) -> AllocatedNormalObjectHandle<'gc> {
            self.normal_object
        }

        fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
            &self.normal_object
        }
    }
}

pub mod stack_trace_element {
    use jvmti_jni_bindings::jint;
    use rust_jvm_common::compressed_classfile::class_names::CClassName;
    use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};


    use crate::{AllocatedHandle, NewJavaValue, WasException};
    use crate::better_java_stack::frames::PushableFrame;
    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter_util::{new_object, run_constructor};
    use crate::java_values::JavaValue;
    use crate::jvm_state::JVMState;
    use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
    use crate::new_java_values::owned_casts::OwnedCastAble;
    use crate::stdlib::java::lang::string::JString;
    use crate::stdlib::java::NewAsObjectOrJavaValue;

    pub struct StackTraceElement<'gc> {
        pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
    }

    impl<'gc> JavaValue<'gc> {
        pub fn cast_stack_trace_element(&self) -> StackTraceElement<'gc> {
            todo!()
            /*StackTraceElement { normal_object: self.unwrap_object_nonnull() }*/
        }
    }

    impl<'gc> StackTraceElement<'gc> {
        pub fn new<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, declaring_class: JString<'gc>, method_name: JString<'gc>, file_name: JString<'gc>, line_number: jint) -> Result<StackTraceElement<'gc>, WasException<'gc>> {
            let class_ = check_initing_or_inited_class(jvm, int_state, CClassName::stack_trace_element().into())?;
            let res = AllocatedHandle::NormalObject(new_object(jvm, int_state, &class_, false));
            let full_args = vec![res.new_java_value(), declaring_class.new_java_value(), method_name.new_java_value(), file_name.new_java_value(), NewJavaValue::Int(line_number)];
            let desc = CMethodDescriptor::void_return(vec![CClassName::string().into(), CClassName::string().into(), CClassName::string().into(), CPDType::IntType]);
            run_constructor(jvm, int_state, class_, full_args, &desc)?;
            Ok(res.cast_stack_trace_element())
        }
    }

    impl<'gc> NewAsObjectOrJavaValue<'gc> for StackTraceElement<'gc> {
        fn object(self) -> AllocatedNormalObjectHandle<'gc> {
            self.normal_object
        }

        fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
            &self.normal_object
        }
    }
}

pub mod member_name {
    use jvmti_jni_bindings::jint;
    use rust_jvm_common::compressed_classfile::class_names::CClassName;
    use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::field_names::FieldName;
    use rust_jvm_common::compressed_classfile::method_names::MethodName;


    use crate::{check_initing_or_inited_class, JavaValueCommon, JVMState, NewJavaValue, NewJavaValueHandle, WasException};
    use crate::better_java_stack::frames::PushableFrame;
    use crate::class_loading::assert_inited_or_initing_class;
    use crate::interpreter_util::{new_object, run_constructor};
    use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
    use crate::new_java_values::owned_casts::OwnedCastAble;
    use crate::stdlib::java::lang::class::JClass;
    use crate::stdlib::java::lang::invoke::method_type::MethodType;
    use crate::stdlib::java::lang::reflect::constructor::Constructor;
    use crate::stdlib::java::lang::reflect::field::Field;
    use crate::stdlib::java::lang::reflect::method::Method;
    use crate::stdlib::java::lang::string::JString;
    use crate::stdlib::java::NewAsObjectOrJavaValue;
    use crate::utils::run_static_or_virtual;

    #[derive(Clone)]
    pub struct MemberName<'gc> {
        pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
    }

    impl<'gc> MemberName<'gc> {
        // private Class<?> clazz;
        // private String name;
        // private Object type;
        // private int flags;
        pub fn get_name_func<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<Option<JString<'gc>>, WasException<'gc>> {
            let member_name_class = assert_inited_or_initing_class(jvm, CClassName::member_name().into());
            let args = vec![self.normal_object.new_java_value()];
            let desc = CMethodDescriptor::empty_args(CClassName::string().into());
            let res = run_static_or_virtual(jvm, int_state, &member_name_class, MethodName::method_getName(), &desc, args)?;
            Ok(res.unwrap().cast_string())
        }

        pub fn is_static<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<bool, WasException<'gc>> {
            let member_name_class = assert_inited_or_initing_class(jvm, CClassName::member_name().into());
            let desc = CMethodDescriptor::empty_args(CPDType::BooleanType);
            let args = vec![self.normal_object.new_java_value()];
            let res = run_static_or_virtual(jvm, int_state, &member_name_class, MethodName::method_isStatic(), &desc, args)?;
            Ok(res.unwrap().as_njv().unwrap_int() != 0)
        }

        pub fn get_name_or_null(&self, jvm: &'gc JVMState<'gc>) -> Option<JString<'gc>> {
            let str_jvalue = self.normal_object.get_var_top_level(jvm, FieldName::field_name());
            Some(str_jvalue.unwrap_object()?.cast_string())
        }

        pub fn get_name(&self, jvm: &'gc JVMState<'gc>) -> JString<'gc> {
            self.get_name_or_null(jvm).unwrap()
        }

        pub fn set_name(&self, jvm: &'gc JVMState<'gc>, new_val: JString<'gc>) {
            self.normal_object.set_var_top_level(jvm, FieldName::field_name(), new_val.new_java_value());
        }

        pub fn get_clazz_or_null(&self, jvm: &'gc JVMState<'gc>) -> Option<JClass<'gc>> {
            let possibly_null = self.normal_object.get_var_top_level(jvm, FieldName::field_clazz());
            Some(possibly_null.unwrap_object()?.cast_class())
        }

        pub fn get_clazz(&self, jvm: &'gc JVMState<'gc>) -> JClass<'gc> {
            self.get_clazz_or_null(jvm).unwrap()
        }

        pub fn set_clazz(&self, jvm: &'gc JVMState<'gc>, new_val: JClass<'gc>) {
            self.normal_object.set_var_top_level(jvm, FieldName::field_clazz(), new_val.new_java_value());
        }

        pub fn set_type(&self, jvm: &'gc JVMState<'gc>, new_val: MethodType<'gc>) {
            self.normal_object.set_var_top_level(jvm, FieldName::field_type(), new_val.new_java_value());
        }

        pub fn get_type(&self, jvm: &'gc JVMState<'gc>) -> NewJavaValueHandle<'gc> {
            self.normal_object.get_var_top_level(jvm, FieldName::field_type())
        }

        pub fn set_flags(&self, jvm: &'gc JVMState<'gc>, new_val: jint) {
            self.normal_object.set_var_top_level(jvm, FieldName::field_flags(), NewJavaValue::Int(new_val));
        }

        pub fn get_flags_or_null(&self, jvm: &'gc JVMState<'gc>) -> Option<jint> {
            Some(self.normal_object.get_var_top_level(jvm, FieldName::field_flags()).unwrap_int())
            /*let maybe_null = self.normal_object.lookup_field(jvm, FieldName::field_flags());
            if maybe_null.try_unwrap_object().is_some() {
                if maybe_null.unwrap_object().is_some() {
                    maybe_null.unwrap_int().into()
                } else {
                    None
                }
            } else {
                maybe_null.unwrap_int().into()
            }*/
        }
        pub fn get_flags(&self, jvm: &'gc JVMState<'gc>) -> jint {
            self.get_flags_or_null(jvm).unwrap()
        }

        pub fn set_resolution(&self, jvm: &'gc JVMState<'gc>, new_val: NewJavaValue<'gc, '_>) {
            self.normal_object.set_var_top_level(jvm, FieldName::field_resolution(), new_val);
        }

        pub fn get_resolution(&self, jvm: &'gc JVMState<'gc>) -> NewJavaValueHandle<'gc> {
            self.normal_object.get_var_top_level(jvm, FieldName::field_resolution())
        }

        pub fn clazz(&self, jvm: &'gc JVMState<'gc>) -> Option<JClass<'gc>> {
            Some(self.normal_object.get_var_top_level(jvm, FieldName::field_clazz()).unwrap_object()?.cast_class())
        }

        pub fn get_method_type<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<MethodType<'gc>, WasException<'gc>> {
            /*let member_name_class = assert_inited_or_initing_class(jvm, CClassName::member_name().into());
            int_state.push_current_operand_stack(JavaValue::Object(self.normal_object.clone().into()));
            run_static_or_virtual(jvm, int_state, &member_name_class, MethodName::method_getMethodType(), &CMethodDescriptor::empty_args(CClassName::method_type().into()), todo!())?;
            Ok(int_state.pop_current_operand_stack(Some(CClassName::method_type().into())).cast_method_type())*/
            todo!()
        }

        pub fn get_field_type<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<Option<JClass<'gc>>, WasException<'gc>> {
            let member_name_class = assert_inited_or_initing_class(jvm, CClassName::member_name().into());
            let args = vec![self.normal_object.new_java_value()];
            let desc = CMethodDescriptor::empty_args(CClassName::class().into());
            let res = run_static_or_virtual(jvm, int_state, &member_name_class, MethodName::method_getFieldType(), &desc, args)?;
            Ok(res.unwrap().cast_class())
        }

        pub fn new_from_field<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, field: Field<'gc>) -> Result<Self, WasException<'gc>> {
            /*let member_class = check_initing_or_inited_class(jvm, int_state, CClassName::member_name().into())?;
            let res = new_object(jvm, int_state, &member_class).to_jv();
            run_constructor(jvm, int_state, member_class, todo!()/*vec![res.clone(), field.java_value()]*/, &CMethodDescriptor::void_return(vec![CClassName::field().into()]))?;
            Ok(res.cast_member_name())*/
            todo!()
        }

        pub fn new_from_method<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, method: Method<'gc>) -> Result<Self, WasException<'gc>> {
            let member_class = check_initing_or_inited_class(jvm, int_state, CClassName::member_name().into())?;
            let res = new_object(jvm, int_state, &member_class, false);
            let desc = CMethodDescriptor::void_return(vec![CClassName::method().into()]);
            run_constructor(jvm, int_state, member_class, vec![res.new_java_value(), method.new_java_value()], &desc)?;
            Ok(res.cast_member_name())
        }

        pub fn new_from_constructor<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, constructor: Constructor<'gc>) -> Result<Self, WasException<'gc>> {
            let member_class = check_initing_or_inited_class(jvm, int_state, CClassName::member_name().into())?;
            let res = new_object(jvm, int_state, &member_class, false);
            let desc = CMethodDescriptor::void_return(vec![CClassName::constructor().into()]);
            let args = vec![res.new_java_value(), constructor.new_java_value()];
            run_constructor(jvm, int_state, member_class, args, &desc)?;
            Ok(res.cast_member_name())
        }
    }

    impl<'gc> NewAsObjectOrJavaValue<'gc> for MemberName<'gc> {
        fn object(self) -> AllocatedNormalObjectHandle<'gc> {
            self.normal_object
        }

        fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
            &self.normal_object
        }
    }
}

pub mod class {
    use std::sync::{Arc, RwLock};

    use runtime_class_stuff::hidden_fields::HiddenJVMField;
    use runtime_class_stuff::RuntimeClass;
    use rust_jvm_common::compressed_classfile::class_names::CClassName;
    use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::field_names::FieldName;
    use rust_jvm_common::compressed_classfile::method_names::MethodName;


    use rust_jvm_common::cpdtype_table::CPDTypeTable;

    use crate::{AllocatedHandle, JavaValueCommon, JVMState, NewJavaValue, WasException};
    use crate::better_java_stack::frames::PushableFrame;
    use crate::class_loading::{check_initing_or_inited_class, ClassIntrinsicsData};
    use crate::interpreter::common::ldc::load_class_constant_by_type;
    use crate::interpreter_util::{new_object, run_constructor};
    use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
    use crate::new_java_values::NewJavaValueHandle;
    use crate::stdlib::java::lang::class_loader::ClassLoader;
    use crate::stdlib::java::lang::string::JString;
    use crate::stdlib::java::NewAsObjectOrJavaValue;
    use crate::utils::run_static_or_virtual;

    pub struct JClass<'gc> {
        normal_object: AllocatedNormalObjectHandle<'gc>,
    }

    impl<'gc> Clone for JClass<'gc> {
        fn clone(&self) -> Self {
            JClass { normal_object: self.normal_object.duplicate_discouraged() }
        }
    }

    impl<'gc> NewJavaValueHandle<'gc> {
        pub fn cast_class(self) -> Option<JClass<'gc>> {
            Some(JClass { normal_object: self.unwrap_object()?.unwrap_normal_object() })
        }
    }

    impl<'gc> AllocatedNormalObjectHandle<'gc> {
        pub fn cast_class(self) -> JClass<'gc> {
            JClass { normal_object: self }
        }
    }

    impl<'gc> AllocatedHandle<'gc> {
        pub fn cast_class(self) -> JClass<'gc> {
            JClass { normal_object: self.unwrap_normal_object() }
        }
    }

    impl<'gc, 'l> NewJavaValue<'gc, 'l> {
        pub fn cast_class(&self) -> Option<JClass<'gc>> {
            Some(JClass { normal_object: self.to_handle_discouraged().unwrap_object_nonnull().unwrap_normal_object() })
        }
    }

    impl<'gc> JClass<'gc> {
        pub fn as_runtime_class(&self, jvm: &'gc JVMState<'gc>) -> Arc<RuntimeClass<'gc>> {
            jvm.classes.read().unwrap().object_to_runtime_class(&self.normal_object)
            //todo I can get rid of this clone since technically only a ref is needed for lookup
        }
        pub fn as_type(&self, jvm: &'gc JVMState<'gc>) -> CPDType {
            self.as_runtime_class(jvm).cpdtype()
        }
    }

    impl<'gc> JClass<'gc> {
        pub fn gc_lifeify(&self) -> JClass<'gc> {
            JClass { normal_object: self.normal_object.clone() }//todo there should be a better way to do this b/c class objects live forever
        }

        pub fn get_class_loader<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<Option<ClassLoader<'gc>>, WasException<'gc>> {
            todo!()
            /*int_state.push_current_operand_stack(JavaValue::Object(self.normal_object.as_allocated_obj().to_gc_managed().clone().into()));
            run_static_or_virtual(jvm, int_state, &self.normal_object.as_allocated_obj().to_gc_managed().unwrap_normal_object().objinfo.class_pointer, MethodName::method_getClassLoader(), &CMethodDescriptor::empty_args(CClassName::classloader().into()), todo!())?;
            Ok(int_state.pop_current_operand_stack(Some(CClassName::object().into())).unwrap_object().map(|cl| JavaValue::Object(cl.into()).cast_class_loader()))*/
        }

        pub fn new<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, loader: Option<ClassLoader<'gc>>, class_intrinsics_data: ClassIntrinsicsData<'gc>) -> Result<Self, WasException<'gc>> {
            let class_class = check_initing_or_inited_class(jvm, int_state, CClassName::class().into())?;
            let will_apply_intrinsic_data = true;
            let res = new_object(jvm, int_state, &class_class, will_apply_intrinsic_data);
            let constructor_desc = CMethodDescriptor::void_return(vec![CClassName::classloader().into()]);
            let loader_njv = match loader.as_ref() {
                None => {
                    NewJavaValue::Null
                }
                Some(loader) => {
                    loader.new_java_value()
                }
            };
            run_constructor(jvm, int_state, class_class.clone(), vec![res.new_java_value(), loader_njv], &constructor_desc)?;
            let res = res.cast_class().apply_intrinsic_data(&class_class, &jvm.cpdtype_table, class_intrinsics_data);
            Ok(res)
        }

        pub(crate) fn apply_intrinsic_data(self, class_class: &Arc<RuntimeClass<'gc>>, cpd_type_table: &RwLock<CPDTypeTable>, class_intrinsics_data: ClassIntrinsicsData<'gc>) -> Self {
            let ClassIntrinsicsData { is_array, is_primitive: _, component_type, this_cpdtype } = class_intrinsics_data;
            let component_type_njv = match component_type.as_ref() {
                None => {
                    NewJavaValue::Null
                }
                Some(component_type) => component_type.new_java_value()
            };
            self.normal_object.set_var_hidden(&class_class, HiddenJVMField::class_is_array(), NewJavaValue::Boolean(u8::from(is_array)));
            self.normal_object.set_var_hidden(&class_class, HiddenJVMField::class_component_type(), component_type_njv);
            let mut cpdtype_guard = cpd_type_table.write().unwrap();
            let this_cpdtype_id = cpdtype_guard.get_cpdtype_id(this_cpdtype).0 as i32;
            let array_wrapped_cpdtype_id = cpdtype_guard.get_cpdtype_id(CPDType::array(this_cpdtype)).0 as i32;
            drop(cpdtype_guard);
            self.normal_object.set_var_hidden(&class_class, HiddenJVMField::class_cpdtype_id(), NewJavaValue::Int(this_cpdtype_id));
            self.normal_object.set_var_hidden(&class_class, HiddenJVMField::class_cpdtype_id_of_wrapped_in_array(), NewJavaValue::Int(array_wrapped_cpdtype_id));
            self
        }

        pub fn debug_assert(&self, jvm: &'gc JVMState<'gc>) {
            let class_class = jvm.classes.read().unwrap().class_class.clone();
            let wrapped_id = self.normal_object.get_var_hidden(jvm, &class_class, HiddenJVMField::class_cpdtype_id_of_wrapped_in_array()).unwrap_int();
            let not_wrapped_id = self.normal_object.get_var_hidden(jvm, &class_class, HiddenJVMField::class_cpdtype_id()).unwrap_int();
            assert_ne!(wrapped_id, not_wrapped_id);
        }

        pub fn from_type<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, ptype: CPDType) -> Result<JClass<'gc>, WasException<'gc>> {
            let res = load_class_constant_by_type(jvm, int_state, ptype)?;
            Ok(res.cast_class().unwrap())//todo we should be able to safely turn handles that live for gc life without reentrant register
        }

        pub fn get_name<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<JString<'gc>, WasException<'gc>> {
            /*int_state.push_current_operand_stack(self.clone().java_value());
            let class_class = check_initing_or_inited_class(jvm, int_state, CClassName::class().into()).unwrap();
            run_static_or_virtual(jvm, int_state, &class_class, MethodName::method_getName(), &CMethodDescriptor::empty_args(CClassName::string().into()), todo!())?;
            let result_popped_from_operand_stack: JavaValue<'gc> = int_state.pop_current_operand_stack(Some(CClassName::string().into()));
            Ok(result_popped_from_operand_stack.cast_string().expect("classes are known to have non-null names"))*/
            todo!()
        }

        pub fn get_generic_interfaces<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<NewJavaValueHandle<'gc>, WasException<'gc>> {
            let class_class = check_initing_or_inited_class(jvm, int_state, CClassName::class().into()).unwrap();
            let args = vec![self.new_java_value()];
            let desc = CMethodDescriptor::empty_args(CPDType::array(CClassName::type_().into()).into());
            let res = run_static_or_virtual(jvm, int_state, &class_class, MethodName::method_getGenericInterfaces(), &desc, args)?.unwrap();
            Ok(res)
        }

        pub fn set_name_(&self, jvm: &'gc JVMState<'gc>, name: JString<'gc>) {
            self.normal_object.set_var_top_level(jvm, FieldName::field_name(), name.new_java_value())
        }

        pub fn object_gc_life(self, jvm: &JVMState<'gc>) -> &'gc AllocatedNormalObjectHandle<'gc> {
            jvm.gc.handle_lives_for_gc_life(self.normal_object)
        }
    }

    impl<'gc> NewAsObjectOrJavaValue<'gc> for JClass<'gc> {
        fn object(self) -> AllocatedNormalObjectHandle<'gc> {
            self.normal_object
        }

        fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
            &self.normal_object
        }
    }
}

pub mod class_loader {
    use rust_jvm_common::compressed_classfile::class_names::CClassName;
    use rust_jvm_common::compressed_classfile::compressed_types::CMethodDescriptor;
    use rust_jvm_common::compressed_classfile::method_names::MethodName;
    use rust_jvm_common::loading::LoaderName;

    use crate::{AllocatedHandle, WasException};
    use crate::better_java_stack::frames::PushableFrame;
    use crate::class_loading::assert_inited_or_initing_class;
    use crate::java_values::JavaValue;
    use crate::jvm_state::JVMState;
    use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
    use crate::new_java_values::NewJavaValueHandle;
    use crate::stdlib::java::lang::class::JClass;
    use crate::stdlib::java::lang::string::JString;
    use crate::stdlib::java::NewAsObjectOrJavaValue;
    use crate::utils::run_static_or_virtual;

    pub struct ClassLoader<'gc> {
        normal_object: AllocatedNormalObjectHandle<'gc>,
    }

    impl Clone for ClassLoader<'_> {
        fn clone(&self) -> Self {
            todo!()
        }
    }

    impl<'gc> JavaValue<'gc> {
        pub fn cast_class_loader(&self) -> ClassLoader<'gc> {
            ClassLoader { normal_object: todo!()/*self.unwrap_object_nonnull()*/ }
        }
    }

    impl<'gc> NewJavaValueHandle<'gc> {
        pub fn cast_class_loader(self) -> ClassLoader<'gc> {
            self.unwrap_object_nonnull().cast_class_loader()
        }
    }

    impl<'gc> AllocatedHandle<'gc> {
        pub fn cast_class_loader(self) -> ClassLoader<'gc> {
            ClassLoader { normal_object: self.unwrap_normal_object() }
        }
    }

    impl<'gc> AllocatedNormalObjectHandle<'gc> {
        pub fn cast_class_loader(self) -> ClassLoader<'gc> {
            ClassLoader { normal_object: self }
        }
    }

    impl<'gc> ClassLoader<'gc> {
        pub fn to_jvm_loader(&self, jvm: &'gc JVMState<'gc>) -> LoaderName {
            let mut classes_guard = jvm.classes.write().unwrap();
            let gc_lifefied_obj = self.normal_object.duplicate_discouraged();
            classes_guard.lookup_or_add_classloader(gc_lifefied_obj)
        }

        pub fn load_class<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, name: JString<'gc>) -> Result<JClass<'gc>, WasException<'gc>> {
            let class_loader = assert_inited_or_initing_class(jvm, CClassName::classloader().into());
            let res = run_static_or_virtual(
                jvm,
                int_state,
                &class_loader,
                MethodName::method_loadClass(),
                &CMethodDescriptor { arg_types: vec![CClassName::string().into()], return_type: CClassName::class().into() },
                vec![self.new_java_value(), name.new_java_value()],
            )?.unwrap();
            Ok(res.cast_class().unwrap())
        }

        /*as_object_or_java_value!();*/
    }

    impl<'gc> NewAsObjectOrJavaValue<'gc> for ClassLoader<'gc> {
        fn object(self) -> AllocatedNormalObjectHandle<'gc> {
            self.normal_object
        }

        fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
            &self.normal_object
        }
    }
}

pub mod string {
    use itertools::Itertools;
    use wtf8::Wtf8Buf;

    use jvmti_jni_bindings::{jchar, jint};
    use rust_jvm_common::compressed_classfile::class_names::CClassName;
    use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::field_names::FieldName;
    use rust_jvm_common::compressed_classfile::method_names::MethodName;


    use crate::{AllocatedHandle, JavaValueCommon, JVMState, NewJavaValue, UnAllocatedObject, WasException};
    use crate::better_java_stack::frames::PushableFrame;
    use crate::class_loading::{assert_inited_or_initing_class, check_initing_or_inited_class};
    use crate::interpreter_util::{new_object, run_constructor};
    use crate::java_values::JavaValue;
    use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
    use crate::new_java_values::NewJavaValueHandle;
    use crate::new_java_values::unallocated_objects::UnAllocatedObjectArray;
    use crate::stdlib::java::NewAsObjectOrJavaValue;
    use crate::utils::run_static_or_virtual;

    pub struct JString<'gc> {
        normal_object: AllocatedNormalObjectHandle<'gc>,
    }

    impl Clone for JString<'_> {
        fn clone(&self) -> Self {
            JString {
                normal_object: self.normal_object.duplicate_discouraged()
            }
        }
    }

    impl<'gc> JavaValue<'gc> {
        pub fn cast_string(&self) -> Option<JString<'gc>> {
            todo!()
            /*Some(JString { normal_object: self.unwrap_object()? })*/
        }
    }

    impl<'gc> NewJavaValueHandle<'gc> {
        pub fn cast_string(self) -> Option<JString<'gc>> {
            Some(JString { normal_object: self.unwrap_object()?.unwrap_normal_object() })
        }
    }

    impl<'gc> AllocatedHandle<'gc> {
        pub fn cast_string(self) -> JString<'gc> {
            JString { normal_object: self.unwrap_normal_object() }
        }
    }

    impl<'gc> JString<'gc> {
        pub fn to_rust_string(&self, jvm: &'gc JVMState<'gc>) -> String {
            let str_obj = &self.normal_object;
            let str_class_pointer = assert_inited_or_initing_class(jvm, CClassName::string().into());
            let temp = str_obj.get_var(jvm, &str_class_pointer, FieldName::field_value());
            let nonnull = temp.unwrap_object_nonnull();
            let chars = nonnull.unwrap_array();
            let borrowed_elems = chars.array_iterator();
            char::decode_utf16(borrowed_elems.map(|jv| jv.unwrap_char_strict())).collect::<Result<String, _>>().expect("really weird string encountered")
        }

        pub fn from_rust(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, rust_str: Wtf8Buf) -> Result<JString<'gc>, WasException<'gc>> {
            let string_class = check_initing_or_inited_class(jvm, int_state, CClassName::string().into()).unwrap(); //todo replace these unwraps
            let string_object = AllocatedHandle::NormalObject(new_object(jvm, int_state, &string_class, false));
            let elems = rust_str.to_ill_formed_utf16().map(|c| NewJavaValue::Char(c as u16)).collect_vec();
            let array_object = UnAllocatedObjectArray {
                whole_array_runtime_class: check_initing_or_inited_class(jvm, int_state, CPDType::array(CPDType::CharType)).unwrap(),
                elems,
            };
            //todo what about check_inited_class for this array type
            let array = NewJavaValueHandle::Object(jvm.allocate_object(UnAllocatedObject::Array(array_object)));
            dbg!(array.as_njv().to_handle_discouraged().unwrap_object_nonnull().unwrap_array().array_iterator().map(|elem| elem.unwrap_char_strict()).collect_vec());
            run_constructor(jvm, int_state, string_class, vec![string_object.new_java_value(), array.as_njv()], &CMethodDescriptor::void_return(vec![CPDType::array(CPDType::CharType)]))?;
            Ok(NewJavaValueHandle::Object(string_object).cast_string().expect("error creating string"))
        }

        pub fn intern<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<JString<'gc>, WasException<'gc>> {
            let string_class = check_initing_or_inited_class(jvm, int_state, CClassName::string().into())?;
            let args = vec![self.new_java_value()];
            let res = run_static_or_virtual(
                jvm,
                int_state,
                &string_class,
                MethodName::method_intern(),
                &CMethodDescriptor::empty_args(CClassName::string().into()),
                args,
            )?.unwrap();
            Ok(res.cast_string().expect("error interning strinng"))
        }

        pub fn value(&self, jvm: &'gc JVMState<'gc>) -> Vec<jchar> {
            let string_class = assert_inited_or_initing_class(jvm, CClassName::string().into());
            let mut res = vec![];
            for elem in self.normal_object.get_var(jvm, &string_class, FieldName::field_value()).unwrap_object_nonnull().unwrap_array().array_iterator() {
                res.push(elem.as_njv().unwrap_char_strict())
            }
            res
        }

        pub fn to_rust_string_better(&self, jvm: &'gc JVMState<'gc>) -> Option<String> {
            let string_class = assert_inited_or_initing_class(jvm, CClassName::string().into());
            let as_allocated_obj = &self.normal_object;
            let value_field = as_allocated_obj.get_var(jvm, &string_class, FieldName::field_value());
            value_field.as_njv().unwrap_object_alloc()?;
            let mut res = vec![];
            for elem in value_field.unwrap_object_nonnull().unwrap_array().array_iterator() {
                res.push(elem.as_njv().unwrap_char_strict())
            }
            String::from_utf16(res.as_slice()).ok()
        }

        pub fn length<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<jint, WasException<'gc>> {
            todo!()/*int_state.push_current_operand_stack(self.clone().java_value())*/;
            let string_class = check_initing_or_inited_class(jvm, int_state, CClassName::string().into())?;
            run_static_or_virtual(jvm, int_state, &string_class, MethodName::method_length(), &CMethodDescriptor::empty_args(CPDType::IntType), todo!())?;
            Ok(todo!()/*int_state.pop_current_operand_stack(Some(CClassName::string().into())).unwrap_int()*/)
        }
    }

    impl<'gc> NewAsObjectOrJavaValue<'gc> for JString<'gc> {
        fn object(self) -> AllocatedNormalObjectHandle<'gc> {
            self.normal_object
        }

        fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
            &self.normal_object
        }
    }
}

pub mod integer {
    use jvmti_jni_bindings::jint;
    use rust_jvm_common::compressed_classfile::field_names::FieldName;


    use crate::{JVMState, StackEntry};
    use crate::java_values::{GcManagedObject, JavaValue};

    pub struct Integer<'gc> {
        normal_object: GcManagedObject<'gc>,
    }

    impl<'gc> JavaValue<'gc> {
        pub fn cast_integer(&self) -> Integer<'gc> {
            Integer { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc> Integer<'gc> {
        pub fn from(_state: &JVMState, _current_frame: &StackEntry, _i: jint) -> Integer<'gc> {
            unimplemented!()
        }

        pub fn value(&self, jvm: &'gc JVMState<'gc>) -> jint {
            self.normal_object.unwrap_normal_object().get_var_top_level(jvm, FieldName::field_value()).unwrap_int()
        }

        //as_object_or_java_value!();
    }
}

pub mod object {
    use crate::{AllocatedHandle, NewAsObjectOrJavaValue, NewJavaValue, NewJavaValueHandle};
    use crate::java_values::JavaValue;
    use crate::new_java_values::allocated_objects::{AllocatedNormalObjectHandle, AllocatedObject};

    pub struct JObject<'gc> {
        pub(crate) normal_object: AllocatedHandle<'gc>,
    }

    impl<'gc> JavaValue<'gc> {
        pub fn cast_object(&self) -> JObject<'gc> {
            JObject { normal_object: todo!()/*self.unwrap_object_nonnull()*/ }
        }
    }

    impl<'gc> AllocatedHandle<'gc> {
        pub fn cast_object(self) -> JObject<'gc> {
            JObject { normal_object: self }
        }
    }

    impl<'gc> NewJavaValueHandle<'gc> {
        pub fn cast_object(self) -> JObject<'gc> {
            JObject { normal_object: self.unwrap_object_nonnull() }
        }
    }


    impl<'gc> NewAsObjectOrJavaValue<'gc> for JObject<'gc> {
        fn object(self) -> AllocatedNormalObjectHandle<'gc> {
            todo!()
        }

        fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
            todo!()
        }

        fn full_object(self) -> AllocatedHandle<'gc> {
            AllocatedHandle::NormalObject(self.object())
        }

        fn full_object_ref(&self) -> AllocatedObject<'gc, '_> {
            self.normal_object.as_allocated_obj()
        }

        fn new_java_value_handle(self) -> NewJavaValueHandle<'gc> {
            NewJavaValueHandle::Object(self.normal_object)
        }

        fn new_java_value(&self) -> NewJavaValue<'gc, '_> {
            self.normal_object.new_java_value()
        }
    }
}

pub mod thread {
    use std::sync::Arc;

    use wtf8::Wtf8Buf;

    use jvmti_jni_bindings::{jboolean, jint};
    use runtime_class_stuff::RuntimeClass;
    use rust_jvm_common::compressed_classfile::class_names::CClassName;
    use rust_jvm_common::compressed_classfile::compressed_descriptors::CompressedMethodDescriptor;
    use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::field_names::FieldName;
    use rust_jvm_common::compressed_classfile::method_names::MethodName;


    use rust_jvm_common::JavaThreadId;

    use crate::{AllocatedHandle, JavaValueCommon, JVMState, NewJavaValue, WasException};
    use crate::better_java_stack::frames::PushableFrame;
    use crate::class_loading::assert_inited_or_initing_class;
    use crate::interpreter_util::{new_object, run_constructor};
    use crate::java_values::JavaValue;
    use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
    use crate::new_java_values::NewJavaValueHandle;
    use crate::stdlib::java::lang::class_loader::ClassLoader;
    use crate::stdlib::java::lang::string::JString;
    use crate::stdlib::java::lang::thread_group::JThreadGroup;
    use crate::stdlib::java::NewAsObjectOrJavaValue;
    use crate::threading::java_thread::JavaThread;
    use crate::utils::run_static_or_virtual;

    pub struct JThread<'gc> {
        normal_object: AllocatedNormalObjectHandle<'gc>,
    }

    impl<'gc> JavaValue<'gc> {
        pub fn cast_thread(&self) -> JThread<'gc> {
            todo!()
        }

        pub fn try_cast_thread(&self) -> Option<JThread<'gc>> {
            todo!()
        }
    }

    impl<'gc> NewJavaValueHandle<'gc> {
        pub fn cast_thread(self) -> JThread<'gc> {
            JThread { normal_object: self.unwrap_object_nonnull().unwrap_normal_object() }
        }

        pub fn try_cast_thread(self) -> Option<JThread<'gc>> {
            Some(JThread { normal_object: self.unwrap_object()?.unwrap_normal_object() }.into())
        }
    }

    impl Clone for JThread<'_> {
        fn clone(&self) -> Self {
            JThread { normal_object: self.normal_object.duplicate_discouraged() }
        }
    }

    impl<'gc> JThread<'gc> {
        pub fn invalid_thread(jvm: &'gc JVMState<'gc>) -> JThread<'gc> {
            todo!()
            /*            JThread {
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
*/
        }

        pub fn tid(&self, jvm: &'gc JVMState<'gc>) -> JavaThreadId {
            let thread_class = assert_inited_or_initing_class(jvm, CClassName::thread().into());
            self.normal_object.get_var(jvm, &thread_class, FieldName::field_tid()).as_njv().unwrap_long_strict()
        }

        pub fn run<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<(), WasException<'gc>> {
            let args = vec![self.normal_object.new_java_value()];
            let thread_class = assert_inited_or_initing_class(jvm, CClassName::thread().into());
            run_static_or_virtual(jvm, int_state, &thread_class, MethodName::method_run(), &CompressedMethodDescriptor::empty_args(CPDType::VoidType), args)?;
            Ok(())
        }

        pub fn exit<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<(), WasException<'gc>> {
            let thread_class = assert_inited_or_initing_class(jvm, CClassName::thread().into());
            let args = vec![self.new_java_value()];
            let desc = CompressedMethodDescriptor::empty_args(CPDType::VoidType);
            run_static_or_virtual(jvm, int_state, &thread_class, MethodName::method_exit(), &desc, args)?;
            Ok(())
        }

        pub fn try_name(&self, jvm: &'gc JVMState<'gc>) -> Option<JString<'gc>> {
            let thread_class = assert_inited_or_initing_class(jvm, CClassName::thread().into());
            self.normal_object.get_var(jvm, &thread_class, FieldName::field_name()).cast_string()
        }

        pub fn name(&self, jvm: &'gc JVMState<'gc>) -> JString<'gc> {
            self.try_name(jvm).unwrap()
        }

        pub fn priority(&self, jvm: &'gc JVMState<'gc>) -> i32 {
            let thread_class = assert_inited_or_initing_class(jvm, CClassName::thread().into());
            todo!()/*self.normal_object.unwrap_normal_object().get_var(jvm, thread_class, FieldName::field_priority()).unwrap_int()*/
        }

        fn top_level_rc(&self) -> Arc<RuntimeClass<'gc>> {
            assert_inited_or_initing_class(&self.normal_object.jvm, CClassName::thread().into())
        }

        fn thread_class(&self) -> Arc<RuntimeClass<'gc>> {
            self.top_level_rc()
        }

        pub fn set_priority(&self, priority: i32) {
            let thread_class = self.thread_class();
            self.normal_object.set_var(&thread_class, FieldName::field_priority(), NewJavaValue::Int(priority));
        }

        pub fn daemon(&self, jvm: &'gc JVMState<'gc>) -> bool {
            let thread_class = assert_inited_or_initing_class(jvm, CClassName::thread().into());
            self.normal_object.get_var(jvm, &thread_class, FieldName::field_daemon()).unwrap_int() != 0
        }

        pub fn set_thread_status(&self, jvm: &'gc JVMState<'gc>, thread_status: jint) {
            let thread_class = assert_inited_or_initing_class(jvm, CClassName::thread().into());
            self.normal_object.set_var(&thread_class, FieldName::field_threadStatus(), NewJavaValue::Int(thread_status));
        }

        pub fn new<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, thread_group: JThreadGroup<'gc>, thread_name: String) -> Result<JThread<'gc>, WasException<'gc>> {
            let thread_class = assert_inited_or_initing_class(jvm, CClassName::thread().into());
            let thread_object = NewJavaValueHandle::Object(AllocatedHandle::NormalObject(new_object(jvm, int_state, &thread_class, false)));
            let thread_name = JString::from_rust(jvm, int_state, Wtf8Buf::from_string(thread_name))?;
            run_constructor(jvm, int_state, thread_class, vec![thread_object.as_njv(), thread_group.new_java_value_handle().as_njv(), thread_name.new_java_value_handle().as_njv()], &CMethodDescriptor::void_return(vec![CClassName::thread_group().into(), CClassName::string().into()]))?;
            Ok(thread_object.cast_thread())
        }

        pub fn get_java_thread(&self, jvm: &'gc JVMState<'gc>) -> Arc<JavaThread<'gc>> {
            self.try_get_java_thread(jvm).unwrap()
        }

        pub fn try_get_java_thread(&self, jvm: &'gc JVMState<'gc>) -> Option<Arc<JavaThread<'gc>>> {
            let tid = self.tid(jvm);
            jvm.thread_state.try_get_thread_by_tid(tid)
        }

        pub fn is_alive<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<jboolean, WasException<'gc>> {
            let thread_class = assert_inited_or_initing_class(jvm, CClassName::thread().into());
            todo!();// int_state.push_current_operand_stack(todo!()/*self.clone().java_value()*/);
            run_static_or_virtual(jvm, int_state, &thread_class, MethodName::method_isAlive(), &CompressedMethodDescriptor::empty_args(CPDType::BooleanType), todo!())?;
            Ok(todo!()/*int_state.pop_current_operand_stack(Some(RuntimeType::IntType)).unwrap_boolean()*/)
        }

        pub fn get_context_class_loader<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<Option<ClassLoader<'gc>>, WasException<'gc>> {
            let thread_class = assert_inited_or_initing_class(jvm, CClassName::thread().into());
            let mut args = vec![];
            args.push(self.new_java_value());
            let res = run_static_or_virtual(
                jvm,
                int_state,
                &thread_class,
                MethodName::method_getContextClassLoader(),
                &CompressedMethodDescriptor::empty_args(CClassName::classloader().into()),
                args,
            )?.unwrap();
            if res.as_njv().unwrap_object().is_none() {
                return Ok(None);
            }
            Ok(res.unwrap_object().unwrap().cast_class_loader().into())
        }

        pub fn get_inherited_access_control_context(&self, jvm: &'gc JVMState<'gc>) -> JThread<'gc> {
            todo!()/*self.normal_object.lookup_field(jvm, FieldName::field_inheritedAccessControlContext()).cast_thread()*/
        }

        // pub fn object(self) -> crate::new_java_values::AllocatedObject<'gc, 'gc> {
        //     todo!()
        // }
        //
        // as_object_or_java_value!();
    }

    impl<'gc> NewAsObjectOrJavaValue<'gc> for JThread<'gc> {
        fn object(self) -> AllocatedNormalObjectHandle<'gc> {
            self.normal_object
        }

        fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
            &self.normal_object
        }
    }
}

pub mod thread_group {
    use std::sync::Arc;

    use jvmti_jni_bindings::{jboolean, jint};
    use runtime_class_stuff::RuntimeClass;
    use rust_jvm_common::compressed_classfile::compressed_types::CMethodDescriptor;


    use crate::{AllocatedHandle, JavaValueCommon, JVMState, WasException};
    use crate::better_java_stack::frames::PushableFrame;
    use crate::interpreter_util::{new_object, run_constructor};
    use crate::java_values::JavaValue;
    use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
    use crate::new_java_values::NewJavaValueHandle;
    use crate::stdlib::java::lang::string::JString;
    use crate::stdlib::java::lang::thread::JThread;
    use crate::stdlib::java::NewAsObjectOrJavaValue;

    pub struct JThreadGroup<'gc> {
        normal_object: AllocatedNormalObjectHandle<'gc>,
    }

    impl<'gc> JavaValue<'gc> {
        pub fn cast_thread_group(&self) -> JThreadGroup<'gc> {
            todo!()
        }

        pub fn try_cast_thread_group(&self) -> Option<JThreadGroup<'gc>> {
            todo!()
        }
    }

    impl<'gc> NewJavaValueHandle<'gc> {
        pub fn cast_thread_group(self) -> JThreadGroup<'gc> {
            JThreadGroup { normal_object: self.unwrap_object_nonnull().unwrap_normal_object() }
        }

        pub fn try_cast_thread_group(self) -> Option<JThreadGroup<'gc>> {
            /*match self.try_unwrap_normal_object() {
                Some(normal_object) => {
                    if normal_object.objinfo.class_pointer.view().name() == CClassName::thread_group().into() {
                        return JThreadGroup { normal_object: self.unwrap_object_nonnull() }.into();
                    }
                    None
                }
                None => None,
            }*/
            todo!()
        }
    }

    impl Clone for JThreadGroup<'_> {
        fn clone(&self) -> Self {
            JThreadGroup { normal_object: self.normal_object.duplicate_discouraged() }
        }
    }

    impl<'gc> JThreadGroup<'gc> {
        pub fn init<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, thread_group_class: Arc<RuntimeClass<'gc>>) -> Result<JThreadGroup<'gc>, WasException<'gc>> {
            let thread_group_object = NewJavaValueHandle::Object(AllocatedHandle::NormalObject(new_object(jvm, int_state, &thread_group_class, false)));
            run_constructor(jvm, int_state, thread_group_class, vec![thread_group_object.as_njv()], &CMethodDescriptor::void_return(vec![]))?;
            Ok(thread_group_object.cast_thread_group())
        }

        pub fn threads(&self, jvm: &'gc JVMState<'gc>) -> Vec<Option<JThread<'gc>>> {
            /*let threads_field = self.normal_object.lookup_field(jvm, FieldName::field_threads());
            let array = threads_field.unwrap_array();
            array
                .array_iterator(jvm)
                .map(|thread| match thread.unwrap_object() {
                    None => None,
                    Some(t) => JavaValue::Object(t.into()).cast_thread().into(),
                })
                .collect()*/
            todo!()
        }

        pub fn threads_non_null(&self, jvm: &'gc JVMState<'gc>) -> Vec<JThread<'gc>> {
            self.threads(jvm).into_iter().flatten().collect()
        }

        pub fn name(&self, jvm: &'gc JVMState<'gc>) -> JString<'gc> {
            /*self.normal_object.lookup_field(jvm, FieldName::field_name()).cast_string().expect("thread group null name")*/
            todo!()
        }

        pub fn daemon(&self, jvm: &'gc JVMState<'gc>) -> jboolean {
            /*self.normal_object.lookup_field(jvm, FieldName::field_daemon()).unwrap_boolean()*/
            todo!()
        }

        pub fn max_priority(&self, jvm: &'gc JVMState<'gc>) -> jint {
            /*self.normal_object.lookup_field(jvm, FieldName::field_maxPriority()).unwrap_int()*/
            todo!()
        }

        pub fn parent(&self, jvm: &'gc JVMState<'gc>) -> Option<JThreadGroup<'gc>> {
            /*self.normal_object.lookup_field(jvm, FieldName::field_parent()).try_cast_thread_group()*/
            todo!()
        }

        // as_object_or_java_value!();
    }

    impl<'gc> NewAsObjectOrJavaValue<'gc> for JThreadGroup<'gc> {
        fn object(self) -> AllocatedNormalObjectHandle<'gc> {
            self.normal_object
        }

        fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
            &self.normal_object
        }
    }
}

pub mod class_not_found_exception {
    use rust_jvm_common::compressed_classfile::class_names::CClassName;
    use rust_jvm_common::compressed_classfile::compressed_types::CMethodDescriptor;
    use crate::{AllocatedHandle, NewAsObjectOrJavaValue, WasException};
    use crate::better_java_stack::frames::PushableFrame;
    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter_util::{new_object_full, run_constructor};
    use crate::jvm_state::JVMState;
    use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
    use crate::stdlib::java::lang::string::JString;

    pub struct ClassNotFoundException<'gc> {
        normal_object: AllocatedNormalObjectHandle<'gc>,
    }

    impl<'gc> AllocatedHandle<'gc> {
        pub fn cast_class_not_found_exception(self) -> ClassNotFoundException<'gc> {
            ClassNotFoundException { normal_object: self.unwrap_normal_object() }
        }
    }

    impl<'gc> ClassNotFoundException<'gc> {
        pub fn new<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, class: JString<'gc>) -> Result<ClassNotFoundException<'gc>, WasException<'gc>> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, CClassName::class_not_found_exception().into())?;
            let this = new_object_full(jvm, int_state, &class_not_found_class);
            run_constructor(jvm, int_state, class_not_found_class, vec![this.new_java_value(), class.new_java_value()], &CMethodDescriptor::void_return(vec![CClassName::string().into()]))?;
            Ok(this.cast_class_not_found_exception())
        }
    }

    impl<'gc> NewAsObjectOrJavaValue<'gc> for ClassNotFoundException<'gc> {
        fn object(self) -> AllocatedNormalObjectHandle<'gc> {
            self.normal_object
        }

        fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
            &self.normal_object
        }
    }
}

pub mod null_pointer_exception {
    use wtf8::Wtf8Buf;
    use rust_jvm_common::compressed_classfile::class_names::CClassName;
    use rust_jvm_common::compressed_classfile::compressed_types::CMethodDescriptor;


    use crate::{NewAsObjectOrJavaValue, NewJavaValueHandle, WasException};
    use crate::better_java_stack::frames::PushableFrame;
    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter_util::{new_object, run_constructor};
    use crate::jvm_state::JVMState;
    use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
    use crate::stdlib::java::lang::string::JString;

    pub struct NullPointerException<'gc> {
        normal_object: AllocatedNormalObjectHandle<'gc>,
    }

    impl<'gc> NewJavaValueHandle<'gc> {
        pub fn cast_null_pointer_exception(self) -> NullPointerException<'gc> {
            NullPointerException { normal_object: self.unwrap_object_nonnull().unwrap_normal_object() }
        }
    }

    impl<'gc> NullPointerException<'gc> {
        pub fn new<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<NullPointerException<'gc>, WasException<'gc>> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, CClassName::null_pointer_exception().into())?;
            let this = new_object(jvm, int_state, &class_not_found_class, false);
            let message = JString::from_rust(jvm, int_state, Wtf8Buf::from_string("This jvm doesn't believe in helpful null pointer messages so you get this instead".to_string()))?;
            let desc = CMethodDescriptor::void_return(vec![CClassName::string().into()]);
            run_constructor(jvm, int_state, class_not_found_class, vec![this.new_java_value(), message.new_java_value()], &desc)?;
            Ok(this.new_java_handle().cast_null_pointer_exception())
        }
    }

    impl<'gc> NewAsObjectOrJavaValue<'gc> for NullPointerException<'gc> {
        fn object(self) -> AllocatedNormalObjectHandle<'gc> {
            self.normal_object
        }

        fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
            &self.normal_object
        }
    }
}

pub mod array_out_of_bounds_exception {
    use jvmti_jni_bindings::jint;
    use rust_jvm_common::compressed_classfile::class_names::CClassName;
    use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};


    use crate::{NewAsObjectOrJavaValue, NewJavaValue, WasException};
    use crate::better_java_stack::frames::PushableFrame;
    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter_util::{new_object, run_constructor};
    use crate::jvm_state::JVMState;
    use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
    use crate::new_java_values::owned_casts::OwnedCastAble;

    pub struct ArrayOutOfBoundsException<'gc> {
        pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
    }

    impl<'gc> ArrayOutOfBoundsException<'gc> {
        pub fn new<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, index: jint) -> Result<ArrayOutOfBoundsException<'gc>, WasException<'gc>> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, CClassName::array_out_of_bounds_exception().into())?;
            let this = new_object(jvm, int_state, &class_not_found_class, false);
            let desc = CMethodDescriptor::void_return(vec![CPDType::IntType]);
            let args = vec![this.new_java_value(), NewJavaValue::Int(index)];
            run_constructor(jvm, int_state, class_not_found_class, args, &desc)?;
            Ok(this.cast_array_out_of_bounds_exception())
        }

        pub fn new_no_index<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<ArrayOutOfBoundsException<'gc>, WasException<'gc>> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, CClassName::array_out_of_bounds_exception().into())?;
            let this = new_object(jvm, int_state, &class_not_found_class, false);
            let desc = CMethodDescriptor::void_return(vec![]);
            run_constructor(jvm, int_state, class_not_found_class, vec![this.new_java_value()], &desc)?;
            Ok(this.cast_array_out_of_bounds_exception())
        }
    }

    impl<'gc> NewAsObjectOrJavaValue<'gc> for ArrayOutOfBoundsException<'gc> {
        fn object(self) -> AllocatedNormalObjectHandle<'gc> {
            self.normal_object
        }

        fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
            &self.normal_object
        }
    }
}

pub mod illegal_argument_exception {
    use rust_jvm_common::compressed_classfile::class_names::CClassName;
    use rust_jvm_common::compressed_classfile::compressed_types::CMethodDescriptor;
    use crate::{AllocatedHandle, NewAsObjectOrJavaValue, pushable_frame_todo, WasException};
    use crate::better_java_stack::frames::PushableFrame;
    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter_util::{new_object_full, run_constructor};
    use crate::java_values::JavaValue;
    use crate::jvm_state::JVMState;
    use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;

    pub struct IllegalArgumentException<'gc> {
        normal_object: AllocatedHandle<'gc>,
    }

    impl<'gc> JavaValue<'gc> {
        pub fn cast_illegal_argument_exception(&self) -> IllegalArgumentException<'gc> {
            IllegalArgumentException { normal_object: todo!()/*self.unwrap_object_nonnull()*/ }
        }
    }

    impl<'gc> AllocatedHandle<'gc> {
        pub fn cast_illegal_argument_exception(self) -> IllegalArgumentException<'gc> {
            IllegalArgumentException { normal_object: self }
        }
    }

    impl<'gc> IllegalArgumentException<'gc> {
        pub fn new<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<IllegalArgumentException<'gc>, WasException<'gc>> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, CClassName::illegal_argument_exception().into())?;
            let this = new_object_full(jvm, pushable_frame_todo()/*int_state*/, &class_not_found_class);
            run_constructor(jvm, int_state, class_not_found_class, vec![this.new_java_value()], &CMethodDescriptor::void_return(vec![]))?;
            Ok(this.cast_illegal_argument_exception())
        }
    }

    impl<'gc> NewAsObjectOrJavaValue<'gc> for IllegalArgumentException<'gc> {
        fn object(self) -> AllocatedNormalObjectHandle<'gc> {
            self.normal_object.unwrap_normal_object()
        }

        fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
            todo!()
        }
    }
}

pub mod long {
    use jvmti_jni_bindings::jlong;
    use rust_jvm_common::compressed_classfile::class_names::CClassName;
    use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::field_names::FieldName;


    use crate::{JavaValueCommon, NewAsObjectOrJavaValue, NewJavaValue, WasException};
    use crate::better_java_stack::frames::PushableFrame;
    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter_util::{new_object, run_constructor};
    use crate::java_values::JavaValue;
    use crate::jvm_state::JVMState;
    use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
    use crate::new_java_values::NewJavaValueHandle;
    use crate::new_java_values::owned_casts::OwnedCastAble;

    pub struct Long<'gc> {
        pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
    }

    impl<'gc> JavaValue<'gc> {
        pub fn cast_long(&self) -> Long<'gc> {
            Long { normal_object: todo!() }
        }
    }

    impl<'gc> NewJavaValueHandle<'gc> {
        pub fn cast_long(&self) -> Long<'gc> {
            Long { normal_object: todo!() }
        }
    }

    impl<'gc> Long<'gc> {
        pub fn new<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, param: jlong) -> Result<Long<'gc>, WasException<'gc>> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, CClassName::long().into())?;
            let this = new_object(jvm, int_state, &class_not_found_class, false);
            let args = vec![this.new_java_value(), NewJavaValue::Long(param)];
            let desc = CMethodDescriptor::void_return(vec![CPDType::LongType]);
            run_constructor(jvm, int_state, class_not_found_class, args, &desc)?;
            Ok(this.cast_long())
        }

        pub fn inner_value(&self, jvm: &'gc JVMState<'gc>) -> jlong {
            self.normal_object.get_var_top_level(jvm, FieldName::field_value()).as_njv().unwrap_long_strict()
        }
    }

    impl<'gc> NewAsObjectOrJavaValue<'gc> for Long<'gc> {
        fn object(self) -> AllocatedNormalObjectHandle<'gc> {
            self.normal_object
        }

        fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
            &self.normal_object
        }
    }
}

pub mod int {
    use jvmti_jni_bindings::jint;
    use rust_jvm_common::compressed_classfile::class_names::CClassName;
    use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::field_names::FieldName;


    use crate::{JavaValueCommon, PushableFrame, WasException};
    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter_util::{new_object, run_constructor};
    use crate::java_values::JavaValue;
    use crate::jvm_state::JVMState;
    use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
    use crate::new_java_values::NewJavaValueHandle;
    use crate::NewAsObjectOrJavaValue;

    pub struct Int<'gc> {
        normal_object: AllocatedNormalObjectHandle<'gc>,
    }

    impl<'gc> JavaValue<'gc> {
        pub fn cast_int(&self) -> Int<'gc> {
            Int { normal_object: todo!()/*self.unwrap_object_nonnull()*/ }
        }
    }

    impl<'gc> NewJavaValueHandle<'gc> {
        pub fn cast_int(self) -> Int<'gc> {
            Int { normal_object: self.unwrap_object().unwrap().unwrap_normal_object() }
        }
    }

    impl<'gc, 'l> Int<'gc> {
        pub fn new<'todo>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut impl PushableFrame<'gc>, param: jint) -> Result<Int<'gc>, WasException<'gc>> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, CClassName::int().into())?;
            let this = new_object(jvm, int_state, &class_not_found_class, false).to_jv();
            run_constructor(jvm, int_state, class_not_found_class, todo!()/*vec![this.clone(), JavaValue::Int(param)]*/, &CMethodDescriptor::void_return(vec![CPDType::IntType]))?;
            /*Ok(this.cast_int())*/
            todo!()
        }

        pub fn inner_value(&self, jvm: &'gc JVMState<'gc>) -> jint {
            self.normal_object.get_var_top_level(jvm, FieldName::field_value()).unwrap_int_strict()
        }
    }

    impl<'gc> NewAsObjectOrJavaValue<'gc> for Int<'gc> {
        fn object(self) -> AllocatedNormalObjectHandle<'gc> {
            todo!()
        }

        fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
            todo!()
        }
    }
}

pub mod short {
    use jvmti_jni_bindings::jshort;
    use rust_jvm_common::compressed_classfile::class_names::CClassName;
    use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::field_names::FieldName;


    use crate::{NewAsObjectOrJavaValue, PushableFrame, WasException};
    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter_util::{new_object, run_constructor};
    use crate::java_values::{GcManagedObject, JavaValue};
    use crate::jvm_state::JVMState;
    use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
    use crate::new_java_values::NewJavaValueHandle;

    pub struct Short<'gc> {
        normal_object: GcManagedObject<'gc>,
    }

    impl<'gc> JavaValue<'gc> {
        pub fn cast_short(&self) -> Short<'gc> {
            Short { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc> NewJavaValueHandle<'gc> {
        pub fn cast_short(&self) -> Short<'gc> {
            Short { normal_object: todo!() }
        }
    }

    impl<'gc> Short<'gc> {
        pub fn new<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, param: jshort) -> Result<Short<'gc>, WasException<'gc>> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, CClassName::short().into())?;
            let this = new_object(jvm, int_state, &class_not_found_class, false).to_jv();
            run_constructor(jvm, int_state, class_not_found_class, todo!()/*vec![this.clone(), NewJavaValue::Short(param)]*/, &CMethodDescriptor::void_return(vec![CPDType::ShortType]))?;
            Ok(this.cast_short())
        }

        pub fn inner_value(&self, jvm: &'gc JVMState<'gc>) -> jshort {
            self.normal_object.lookup_field(jvm, FieldName::field_value()).unwrap_short()
        }
    }

    impl<'gc> NewAsObjectOrJavaValue<'gc> for Short<'gc> {
        fn object(self) -> AllocatedNormalObjectHandle<'gc> {
            todo!()
        }

        fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
            todo!()
        }
    }
}

pub mod byte {
    use jvmti_jni_bindings::jbyte;
    use rust_jvm_common::compressed_classfile::class_names::CClassName;
    use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::field_names::FieldName;


    use crate::{NewAsObjectOrJavaValue, PushableFrame, WasException};
    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter_util::{new_object, run_constructor};
    use crate::java_values::{GcManagedObject, JavaValue};
    use crate::jvm_state::JVMState;
    use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
    use crate::new_java_values::NewJavaValueHandle;

    pub struct Byte<'gc> {
        normal_object: GcManagedObject<'gc>,
    }

    impl<'gc> JavaValue<'gc> {
        pub fn cast_byte(&self) -> Byte<'gc> {
            Byte { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc> NewJavaValueHandle<'gc> {
        pub fn cast_byte(&self) -> Byte<'gc> {
            Byte { normal_object: todo!() }
        }
    }

    impl<'gc> Byte<'gc> {
        pub fn new<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, param: jbyte) -> Result<Byte<'gc>, WasException<'gc>> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, CClassName::byte().into())?;
            let this = new_object(jvm, int_state, &class_not_found_class, false).to_jv();
            run_constructor(jvm, int_state, class_not_found_class, todo!()/*vec![this.clone(), JavaValue::Byte(param)]*/, &CMethodDescriptor::void_return(vec![CPDType::ByteType]))?;
            Ok(this.cast_byte())
        }

        pub fn inner_value(&self, jvm: &'gc JVMState<'gc>) -> jbyte {
            self.normal_object.lookup_field(jvm, FieldName::field_value()).unwrap_byte()
        }
    }

    impl<'gc> NewAsObjectOrJavaValue<'gc> for Byte<'gc> {
        fn object(self) -> AllocatedNormalObjectHandle<'gc> {
            todo!()
        }

        fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
            todo!()
        }
    }
}

pub mod boolean {
    use jvmti_jni_bindings::jboolean;
    use rust_jvm_common::compressed_classfile::class_names::CClassName;
    use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::field_names::FieldName;


    use crate::{NewAsObjectOrJavaValue, PushableFrame, WasException};
    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter_util::{new_object, run_constructor};
    use crate::java_values::{GcManagedObject, JavaValue};
    use crate::jvm_state::JVMState;
    use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
    use crate::new_java_values::NewJavaValueHandle;

    pub struct Boolean<'gc> {
        normal_object: GcManagedObject<'gc>,
    }

    impl<'gc> JavaValue<'gc> {
        pub fn cast_boolean(&self) -> Boolean<'gc> {
            Boolean { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc> NewJavaValueHandle<'gc> {
        pub fn cast_boolean(&self) -> Boolean<'gc> {
            Boolean { normal_object: todo!() }
        }
    }

    impl<'gc> Boolean<'gc> {
        //as_object_or_java_value!();

        pub fn new<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, param: jboolean) -> Result<Boolean<'gc>, WasException<'gc>> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, CClassName::boolean().into())?;
            let this = new_object(jvm, int_state, &class_not_found_class, false).to_jv();
            run_constructor(jvm, int_state, class_not_found_class, todo!()/*vec![this.clone(), JavaValue::Boolean(param)]*/, &CMethodDescriptor::void_return(vec![CPDType::BooleanType]))?;
            Ok(this.cast_boolean())
        }

        pub fn inner_value(&self, jvm: &'gc JVMState<'gc>) -> jboolean {
            self.normal_object.lookup_field(jvm, FieldName::field_value()).unwrap_boolean()
        }
    }

    impl<'gc> NewAsObjectOrJavaValue<'gc> for Boolean<'gc> {
        fn object(self) -> AllocatedNormalObjectHandle<'gc> {
            todo!()
        }

        fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
            todo!()
        }
    }
}

pub mod char {
    use jvmti_jni_bindings::jchar;
    use rust_jvm_common::compressed_classfile::class_names::CClassName;
    use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::field_names::FieldName;


    use crate::{NewAsObjectOrJavaValue, PushableFrame, WasException};
    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter_util::{new_object, run_constructor};
    use crate::java_values::{GcManagedObject, JavaValue};
    use crate::jvm_state::JVMState;
    use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
    use crate::new_java_values::NewJavaValueHandle;

    pub struct Char<'gc> {
        normal_object: GcManagedObject<'gc>,
    }

    impl<'gc> JavaValue<'gc> {
        pub fn cast_char(&self) -> Char<'gc> {
            Char { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc> NewJavaValueHandle<'gc> {
        pub fn cast_char(&self) -> Char<'gc> {
            Char { normal_object: todo!() }
        }
    }

    impl<'gc> Char<'gc> {
        pub fn new<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, param: jchar) -> Result<Char<'gc>, WasException<'gc>> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, CClassName::character().into())?;
            let this = new_object(jvm, int_state, &class_not_found_class, false).to_jv();
            run_constructor(jvm, int_state, class_not_found_class, todo!()/*vec![this.clone(), JavaValue::Char(param)]*/, &CMethodDescriptor::void_return(vec![CPDType::CharType]))?;
            Ok(this.cast_char())
        }

        pub fn inner_value(&self, jvm: &'gc JVMState<'gc>) -> jchar {
            self.normal_object.lookup_field(jvm, FieldName::field_value()).unwrap_char()
        }
    }

    impl<'gc> NewAsObjectOrJavaValue<'gc> for Char<'gc> {
        fn object(self) -> AllocatedNormalObjectHandle<'gc> {
            todo!()
        }

        fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
            todo!()
        }
    }
}

pub mod float {
    use jvmti_jni_bindings::jfloat;
    use rust_jvm_common::compressed_classfile::class_names::CClassName;
    use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::field_names::FieldName;


    use crate::{NewAsObjectOrJavaValue, PushableFrame, WasException};
    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter_util::{new_object, run_constructor};
    use crate::java_values::{GcManagedObject, JavaValue};
    use crate::jvm_state::JVMState;
    use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
    use crate::new_java_values::NewJavaValueHandle;

    pub struct Float<'gc> {
        normal_object: GcManagedObject<'gc>,
    }

    impl<'gc> JavaValue<'gc> {
        pub fn cast_float(&self) -> Float<'gc> {
            Float { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc> NewJavaValueHandle<'gc> {
        pub fn cast_float(&self) -> Float<'gc> {
            Float { normal_object: todo!() }
        }
    }

    impl<'gc> Float<'gc> {
        pub fn new<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, param: jfloat) -> Result<Float<'gc>, WasException<'gc>> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, CClassName::float().into())?;
            let this = new_object(jvm, int_state, &class_not_found_class, false).to_jv();
            run_constructor(jvm, int_state, class_not_found_class, todo!()/*vec![this.clone(), JavaValue::Float(param)]*/, &CMethodDescriptor::void_return(vec![CPDType::FloatType]))?;
            Ok(this.cast_float())
        }

        pub fn inner_value(&self, jvm: &'gc JVMState<'gc>) -> jfloat {
            self.normal_object.lookup_field(jvm, FieldName::field_value()).unwrap_float()
        }
    }

    impl<'gc> NewAsObjectOrJavaValue<'gc> for Float<'gc> {
        fn object(self) -> AllocatedNormalObjectHandle<'gc> {
            todo!()
        }

        fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
            todo!()
        }
    }
}

pub mod double {
    use jvmti_jni_bindings::jdouble;
    use rust_jvm_common::compressed_classfile::class_names::CClassName;
    use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::field_names::FieldName;


    use crate::{NewAsObjectOrJavaValue, WasException};
    use crate::better_java_stack::frames::PushableFrame;
    use crate::class_loading::check_initing_or_inited_class;
    use crate::interpreter_util::{new_object, run_constructor};
    use crate::java_values::{GcManagedObject, JavaValue};
    use crate::jvm_state::JVMState;
    use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
    use crate::new_java_values::NewJavaValueHandle;

    pub struct Double<'gc> {
        normal_object: GcManagedObject<'gc>,
    }

    impl<'gc> JavaValue<'gc> {
        pub fn cast_double(&self) -> Double<'gc> {
            Double { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc> NewJavaValueHandle<'gc> {
        pub fn cast_double(&self) -> Double<'gc> {
            Double { normal_object: todo!() }
        }
    }

    impl<'gc> Double<'gc> {
        pub fn new<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, param: jdouble) -> Result<Double<'gc>, WasException<'gc>> {
            let class_not_found_class = check_initing_or_inited_class(jvm, int_state, CClassName::double().into())?;
            let this = new_object(jvm, int_state, &class_not_found_class, false).to_jv();
            run_constructor(jvm, int_state, class_not_found_class, todo!()/*vec![this.clone(), JavaValue::Double(param)]*/, &CMethodDescriptor::void_return(vec![CPDType::DoubleType]))?;
            Ok(this.cast_double())
        }

        pub fn inner_value(&self, jvm: &'gc JVMState<'gc>) -> jdouble {
            self.normal_object.lookup_field(jvm, FieldName::field_value()).unwrap_double()
        }
    }

    impl<'gc> NewAsObjectOrJavaValue<'gc> for Double<'gc> {
        fn object(self) -> AllocatedNormalObjectHandle<'gc> {
            todo!()
        }

        fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
            todo!()
        }
    }
}


pub mod reflect;
pub mod system;