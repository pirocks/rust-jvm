use std::sync::Arc;

use jvmti_jni_bindings::jint;
use runtime_class_stuff::RuntimeClass;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};
use rust_jvm_common::compressed_classfile::field_names::FieldName;
use rust_jvm_common::compressed_classfile::method_names::MethodName;


use crate::{AllocatedHandle, JavaValueCommon, JVMState, NewJavaValue, NewJavaValueHandle, WasException};
use crate::better_java_stack::frames::PushableFrame;
use crate::class_loading::assert_inited_or_initing_class;
use crate::interpreter_util::new_object;
use crate::java_values::JavaValue;
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
use crate::new_java_values::owned_casts::OwnedCastAble;
use crate::stdlib::java::lang::class::JClass;
use crate::stdlib::java::lang::class_loader::ClassLoader;
use crate::stdlib::java::lang::invoke::method_type_form::MethodTypeForm;
use crate::stdlib::java::lang::string::JString;
use crate::stdlib::java::NewAsObjectOrJavaValue;
use crate::utils::run_static_or_virtual;

#[derive(Clone)]
pub struct MethodType<'gc> {
    pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
}

impl<'gc> MethodType<'gc> {
    pub fn from_method_descriptor_string<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, str: JString<'gc>, class_loader: Option<ClassLoader<'gc>>) -> Result<MethodType<'gc>, WasException<'gc>> {
        let method_type: Arc<RuntimeClass<'gc>> = assert_inited_or_initing_class(jvm, CClassName::method_type().into());
        let desc = CMethodDescriptor {
            arg_types: vec![CClassName::string().into(), CClassName::classloader().into()],
            return_type: CClassName::method_type().into(),
        };
        let res = run_static_or_virtual(
            jvm,
            int_state,
            &method_type,
            MethodName::method_fromMethodDescriptorString(),
            &desc,
            vec![str.new_java_value(), class_loader.as_ref().map(|x| x.new_java_value()).unwrap_or(NewJavaValue::Null)],
        )?.unwrap();
        Ok(res.cast_method_type())
    }

    pub fn set_rtype(&self, jvm: &'gc JVMState<'gc>, rtype: JClass<'gc>) {
        self.normal_object.set_var_top_level(jvm, FieldName::field_rtype(), rtype.new_java_value());
    }

    pub fn get_rtype_or_null<'k>(&self, jvm: &'gc JVMState<'gc>) -> Option<JClass<'gc>> {
        Some(self.normal_object.get_var_top_level(jvm, FieldName::field_rtype()).unwrap_object()?.cast_class())
        /*if maybe_null.try_unwrap_object().is_some() {
            if maybe_null.unwrap_object().is_some() {
                maybe_null.to_new().cast_class().into()
            } else {
                None
            }
        } else {
            maybe_null.to_new().cast_class().into()
        }*/
    }
    pub fn get_rtype<'k>(&self, jvm: &'gc JVMState<'gc>) -> JClass<'gc> {
        self.get_rtype_or_null(jvm).unwrap()
    }

    pub fn get_rtype_as_type(&self, jvm: &'gc JVMState<'gc>) -> CPDType {
        self.get_rtype(jvm).as_type(jvm)
    }

    pub fn set_ptypes<'irrelevant>(&self, jvm: &'gc JVMState<'gc>, ptypes: NewJavaValue<'gc, 'irrelevant>) {
        self.normal_object.set_var_top_level(jvm, FieldName::field_ptypes(), ptypes.as_njv());
    }

    pub fn get_ptypes_or_null(&self, jvm: &'gc JVMState<'gc>) -> Option<NewJavaValueHandle<'gc>> {
        Some(self.normal_object.get_var_top_level(jvm, FieldName::field_ptypes()).unwrap_object()?.new_java_value_handle())
        /*if maybe_null.try_unwrap_object().is_some() {
            if maybe_null.unwrap_object().is_some() {
                maybe_null.clone().into()
            } else {
                None
            }
        } else {
            maybe_null.clone().into()
        }*/
    }
    pub fn get_ptypes(&self, jvm: &'gc JVMState<'gc>) -> NewJavaValueHandle<'gc> {
        self.get_ptypes_or_null(jvm).unwrap()
    }

    pub fn get_ptypes_as_types(&self, jvm: &'gc JVMState<'gc>) -> Vec<CPDType> {
        self.get_ptypes(jvm).unwrap_object_nonnull().unwrap_array().array_iterator().map(|x| x.cast_class().unwrap().as_type(jvm)).collect()
    }

    pub fn set_form(&self, jvm: &'gc JVMState<'gc>, form: MethodTypeForm<'gc>) {
        self.normal_object.set_var_top_level(jvm, FieldName::field_form(), form.new_java_value());
    }

    pub fn get_form(&self, jvm: &'gc JVMState<'gc>) -> MethodTypeForm<'gc> {
        self.normal_object.get_var_top_level(jvm, FieldName::field_form()).cast_method_type_form()
    }

    pub fn set_wrap_alt(&self, jvm: &'gc JVMState<'gc>, val: JavaValue<'gc>) {
        self.normal_object.set_var_top_level(jvm, FieldName::field_ptypes(), val.to_new());
    }

    pub fn set_invokers(&self, jvm: &'gc JVMState<'gc>, invokers: JavaValue<'gc>) {
        self.normal_object.set_var_top_level(jvm, FieldName::field_invokers(), invokers.to_new());
    }

    pub fn set_method_descriptors(&self, jvm: &'gc JVMState<'gc>, method_descriptor: JavaValue<'gc>) {
        self.normal_object.set_var_top_level(jvm, FieldName::field_methodDescriptor(), method_descriptor.to_new());
    }

    pub fn parameter_type<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, int: jint) -> Result<JClass<'gc>, WasException<'gc>> {
        let method_type = assert_inited_or_initing_class(jvm, CClassName::method_type().into());
        let desc = CMethodDescriptor { arg_types: vec![CPDType::IntType], return_type: CClassName::class().into() };
        let args = vec![self.new_java_value(), NewJavaValue::Int(int)];
        let res = run_static_or_virtual(jvm, int_state, &method_type, MethodName::method_parameterType(), &desc, args)?;
        Ok(res.unwrap().cast_class().unwrap())
    }

    pub fn new<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, _rtype: JClass<'gc>, _ptypes: Vec<JClass<'gc>>, _form: MethodTypeForm<'gc>, _wrap_alt: JavaValue<'gc>, _invokers: JavaValue<'gc>, _method_descriptor: JavaValue<'gc>) -> MethodType<'gc> {
        let method_type: Arc<RuntimeClass<'gc>> = assert_inited_or_initing_class(jvm, CClassName::method_type().into());
        let res_handle: AllocatedNormalObjectHandle<'gc> = new_object(jvm, int_state, &method_type, false);
        let _res = AllocatedHandle::NormalObject(res_handle).cast_method_type();
        let _ptypes_arr_handle = jvm.allocate_object(todo!()/*Object::Array(ArrayObject {
            // elems: UnsafeCell::new(ptypes.into_iter().map(|x| x.java_value().to_native()).collect::<Vec<_>>()),
            whole_array_runtime_class: todo!(),
            loader: todo!(),
            len: todo!(),
            elems_base: todo!(),
            phantom_data: Default::default(),
            elem_type: CClassName::class().into(),
            // monitor: jvm.thread_state.new_monitor("".to_string()),
        })*/);
        let ptypes_arr = _ptypes_arr_handle.new_java_value();
        _res.set_ptypes(jvm, ptypes_arr);
        _res.set_rtype(jvm, _rtype);
        _res.set_form(jvm, _form);
        _res.set_wrap_alt(jvm, _wrap_alt);
        _res.set_invokers(jvm, _invokers);
        _res.set_method_descriptors(jvm, _method_descriptor);
        _res
    }


}

impl<'gc> NewAsObjectOrJavaValue<'gc> for MethodType<'gc> {
    fn object(self) -> AllocatedNormalObjectHandle<'gc> {
        self.normal_object
    }

    fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
        &self.normal_object
    }
}
