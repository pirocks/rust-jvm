use wtf8::Wtf8Buf;

use classfile_view::view::ClassView;
use classfile_view::view::method_view::MethodView;
use jvmti_jni_bindings::jint;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};
use rust_jvm_common::compressed_classfile::field_names::FieldName;
use rust_jvm_common::compressed_classfile::method_names::MethodName;


use crate::{JavaValueCommon, NewJavaValue, WasException};
use crate::better_java_stack::frames::PushableFrame;
use crate::class_loading::check_initing_or_inited_class;
use crate::interpreter::common::ldc::load_class_constant_by_type;
use crate::interpreter_util::{new_object_full, run_constructor};
use crate::java_values::JavaValue;
use crate::jvm_state::JVMState;
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
use crate::new_java_values::NewJavaValueHandle;
use crate::new_java_values::owned_casts::OwnedCastAble;
use crate::stdlib::java::lang::class::JClass;
use crate::stdlib::java::lang::reflect::{exception_types_table, get_modifiers, get_signature, parameters_type_objects};
use crate::stdlib::java::lang::string::JString;
use crate::stdlib::java::NewAsObjectOrJavaValue;

pub struct Method<'gc> {
    pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
}

impl<'gc> JavaValue<'gc> {
    pub fn cast_method(&self) -> Method<'gc> {
        todo!()
        /*Method { normal_object: self.unwrap_object_nonnull() }*/
    }
}

impl<'gc> Method<'gc> {
    pub fn method_object_from_method_view<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, method_view: &MethodView) -> Result<Method<'gc>, WasException<'gc>> {
        let clazz = {
            let field_class_type = method_view.classview().type_();
            //todo so if we are calling this on int.class that is caught by the unimplemented above.
            load_class_constant_by_type(jvm, int_state, field_class_type)?.cast_class().unwrap()
        };
        let name = {
            let name = method_view.name();
            if name == MethodName::constructor_init() {
                todo!()
                // return Ok(Constructor::constructor_object_from_method_view(jvm, int_state, method_view)?.java_value().cast_method());
            }
            JString::from_rust(jvm, int_state, Wtf8Buf::from_string(name.0.to_str(&jvm.string_pool)))?.intern(jvm, int_state)?
        };
        let parameter_types = parameters_type_objects(jvm, int_state, &method_view)?;
        let return_type = {
            let cpdtype = method_view.desc().return_type.clone(); //todo this is a spurious clone
            JClass::from_type(jvm, int_state, cpdtype)?
        };
        let exception_types = exception_types_table(jvm, int_state, &method_view)?;
        let modifiers = get_modifiers(&method_view);
        //todo what does slot do?
        let slot = -1;
        let signature = get_signature(jvm, int_state, &method_view)?;
        let byte_array_rc = check_initing_or_inited_class(jvm, int_state, CPDType::array(CPDType::ByteType)).unwrap();
        let annotations = NewJavaValueHandle::from_optional_object(method_view.get_annotation_bytes().map(|param_annotations| {
            JavaValue::byte_array(jvm, int_state, param_annotations).unwrap()
        }));
        let parameter_annotations = NewJavaValueHandle::from_optional_object(method_view.get_parameter_annotation_bytes().map(|param_annotations| {
            JavaValue::byte_array(jvm, int_state, param_annotations).unwrap()
        }));
        let annotation_default = NewJavaValueHandle::from_optional_object(method_view.get_annotation_default_bytes().map(|default_annotation_bytes| {
            JavaValue::byte_array(jvm, int_state, default_annotation_bytes).unwrap()
        }));
        Ok(Method::new_method(jvm, int_state, clazz, name, parameter_types, return_type, exception_types, modifiers, slot, signature, annotations, parameter_annotations, annotation_default)?)
    }

    pub fn new_method<'l>(
        jvm: &'gc JVMState<'gc>,
        int_state: &mut impl PushableFrame<'gc>,
        clazz: JClass<'gc>,
        name: JString<'gc>,
        parameter_types: NewJavaValueHandle<'gc>,
        return_type: JClass<'gc>,
        exception_types: NewJavaValueHandle<'gc>,
        modifiers: jint,
        slot: jint,
        signature: Option<JString<'gc>>,
        annotations: NewJavaValueHandle<'gc>,
        parameter_annotations: NewJavaValueHandle<'gc>,
        annotation_default: NewJavaValueHandle<'gc>,
    ) -> Result<Method<'gc>, WasException<'gc>> {
        let method_class = check_initing_or_inited_class(jvm, int_state, CClassName::method().into()).unwrap();
        let method_object = new_object_full(jvm, int_state, &method_class);
        let full_args = vec![method_object.new_java_value(),
                             clazz.new_java_value(),
                             name.new_java_value(),
                             parameter_types.as_njv(),
                             return_type.new_java_value(),
                             exception_types.as_njv(),
                             NewJavaValue::Int(modifiers),
                             NewJavaValue::Int(slot),
                             signature.as_ref().map(|jstring| jstring.new_java_value()).unwrap_or(NewJavaValue::Null),
                             annotations.as_njv(),
                             parameter_annotations.as_njv(),
                             annotation_default.as_njv(), ];
        //todo replace with wrapper object
        let c_method_descriptor = CMethodDescriptor::void_return(vec![
            CClassName::class().into(),
            CClassName::string().into(),
            CPDType::array(CClassName::class().into()),
            CClassName::class().into(),
            CPDType::array(CClassName::class().into()),
            CPDType::IntType,
            CPDType::IntType,
            CClassName::string().into(),
            CPDType::array(CPDType::ByteType),
            CPDType::array(CPDType::ByteType),
            CPDType::array(CPDType::ByteType),
        ]);
        run_constructor(jvm, int_state, method_class, full_args, &c_method_descriptor)?;
        Ok(method_object.cast_method())
    }

    pub fn get_clazz(&self, jvm: &'gc JVMState<'gc>) -> JClass<'gc> {
        self.normal_object.get_var_top_level(jvm, FieldName::field_clazz()).cast_class().unwrap()
        //todo this unwrap
    }

    pub fn get_modifiers(&self, jvm: &'gc JVMState<'gc>) -> jint {
        self.normal_object.get_var_top_level(jvm, FieldName::field_modifiers()).unwrap_int()
    }

    pub fn get_name(&self, jvm: &'gc JVMState<'gc>) -> JString<'gc> {
        todo!()
        /*self.normal_object.lookup_field(jvm, FieldName::field_name()).cast_string().expect("methods must have names")*/
    }

    pub fn parameter_types(&self, jvm: &'gc JVMState<'gc>) -> Vec<JClass<'gc>> {
        self.normal_object.get_var_top_level(jvm, FieldName::field_parameterTypes())
            .unwrap_object()
            .unwrap()
            .unwrap_array()
            .array_iterator()
            .map(|value| value.cast_class().unwrap())
            .collect()
    }

    pub fn get_slot_or_null(&self, jvm: &'gc JVMState<'gc>) -> Option<jint> {
        todo!()
        /*let maybe_null = self.normal_object.lookup_field(jvm, FieldName::field_slot());
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
    pub fn get_slot(&self, jvm: &'gc JVMState<'gc>) -> jint {
        todo!()
        /*self.get_slot_or_null(jvm).unwrap()*/
    }
    pub fn get_return_type_or_null(&self, jvm: &'gc JVMState<'gc>) -> Option<JClass<'gc>> {
        self.normal_object.get_var_top_level(jvm, FieldName::field_returnType()).cast_class()
    }
    pub fn get_return_type(&self, jvm: &'gc JVMState<'gc>) -> JClass<'gc> {
        self.get_return_type_or_null(jvm).unwrap()
    }

    // as_object_or_java_value!();
}

impl<'gc> NewAsObjectOrJavaValue<'gc> for Method<'gc> {
    fn object(self) -> AllocatedNormalObjectHandle<'gc> {
        self.normal_object
    }

    fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
        &self.normal_object
    }
}
