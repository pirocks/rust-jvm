use classfile_view::view::ClassView;
use classfile_view::view::method_view::MethodView;
use jvmti_jni_bindings::jint;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};
use rust_jvm_common::compressed_classfile::field_names::FieldName;


use crate::{JavaValueCommon, NewJavaValue, PushableFrame, WasException};
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

pub struct Constructor<'gc> {
    pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
}

impl<'gc> JavaValue<'gc> {
    pub fn cast_constructor(&self) -> Constructor<'gc> {
        todo!()
        /*Constructor { normal_object: self.unwrap_object_nonnull() }*/
    }
}

impl<'gc> Constructor<'gc> {
    pub fn constructor_object_from_method_view<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, method_view: &MethodView) -> Result<Constructor<'gc>, WasException<'gc>> {
        let clazz = {
            let field_class_type = method_view.classview().type_();
            //todo this doesn't cover the full generality of this, b/c we could be calling on int.class or array classes
            load_class_constant_by_type(jvm, int_state, field_class_type)?.cast_class().unwrap()
        };

        let parameter_types = parameters_type_objects(jvm, int_state, &method_view)?;
        let exception_types = exception_types_table(jvm, int_state, &method_view)?;
        let modifiers = get_modifiers(&method_view);
        //todo what does slot do?
        let slot = -1;
        let signature = get_signature(jvm, int_state, &method_view)?;
        let annotations = NewJavaValueHandle::from_optional_object(method_view.get_annotation_bytes().map(|annotations| {
            JavaValue::byte_array(jvm, int_state, annotations).unwrap()
        }));
        let parameter_annotations = NewJavaValueHandle::from_optional_object(method_view.get_parameter_annotation_bytes().map(|param_annotations| {
            JavaValue::byte_array(jvm, int_state, param_annotations).unwrap()
        }));
        Constructor::new_constructor(jvm, int_state, clazz, parameter_types.as_njv(), exception_types.as_njv(), modifiers, slot, signature, annotations, parameter_annotations)
    }

    pub fn new_constructor<'l>(
        jvm: &'gc JVMState<'gc>,
        int_state: &mut impl PushableFrame<'gc>,
        clazz: JClass<'gc>,
        parameter_types: NewJavaValue<'gc, '_>,
        exception_types: NewJavaValue<'gc, '_>,
        modifiers: jint,
        slot: jint,
        signature: Option<JString<'gc>>,
        annotations: NewJavaValueHandle<'gc>,
        parameter_annotations: NewJavaValueHandle<'gc>,
    ) -> Result<Constructor<'gc>, WasException<'gc>> {
        let constructor_class = check_initing_or_inited_class(jvm, int_state, CClassName::constructor().into())?;
        let constructor_object = new_object_full(jvm, int_state, &constructor_class);

        //todo impl annotations
        let empty_byte_array_rc = check_initing_or_inited_class(jvm, int_state, CPDType::array(CPDType::ByteType)).unwrap();
        let empty_byte_array = NewJavaValueHandle::empty_byte_array(jvm, empty_byte_array_rc);
        let full_args = vec![constructor_object.new_java_value(),
                             clazz.new_java_value(),
                             parameter_types,
                             exception_types,
                             NewJavaValue::Int(modifiers),
                             NewJavaValue::Int(slot),
                             signature.as_ref().map(|jstring| jstring.new_java_value()).unwrap_or(NewJavaValue::Null),
                             annotations.as_njv(),
                             parameter_annotations.as_njv()];
        let c_method_descriptor = CMethodDescriptor::void_return(vec![CClassName::class().into(), CPDType::array(CClassName::class().into()), CPDType::array(CClassName::class().into()), CPDType::IntType, CPDType::IntType, CClassName::string().into(), CPDType::array(CPDType::ByteType), CPDType::array(CPDType::ByteType)]);
        run_constructor(jvm, int_state, constructor_class, full_args, &c_method_descriptor)?;
        Ok(constructor_object.cast_constructor())
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
        self.normal_object.get_var_top_level(jvm, FieldName::field_parameterTypes()).unwrap_object().unwrap().unwrap_array().array_iterator().map(|value| value.cast_class().unwrap()).collect()
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
        todo!()
        /*let maybe_null = self.normal_object.lookup_field(jvm, FieldName::field_returnType());
        if maybe_null.try_unwrap_object().is_some() {
            if maybe_null.unwrap_object().is_some() {
                maybe_null.to_new().cast_class().into()
            } else {
                None
            }
        } else {
            maybe_null.to_new().cast_class().into()
        }*/
    }
    pub fn get_return_type(&self, jvm: &'gc JVMState<'gc>) -> JClass<'gc> {
        todo!()
        /*self.get_return_type_or_null(jvm).unwrap()*/
    }

    /*as_object_or_java_value!();*/
}

impl<'gc> NewAsObjectOrJavaValue<'gc> for Constructor<'gc> {
    fn object(self) -> AllocatedNormalObjectHandle<'gc> {
        self.normal_object
    }

    fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
        &self.normal_object
    }
}
