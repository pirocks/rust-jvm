use std::sync::{Arc, RwLock};

use runtime_class_stuff::hidden_fields::HiddenJVMField;
use runtime_class_stuff::RuntimeClass;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};
use rust_jvm_common::compressed_classfile::field_names::FieldName;
use rust_jvm_common::compressed_classfile::method_names::MethodName;


use rust_jvm_common::cpdtype_table::CPDTypeTable;

use crate::{JavaValueCommon, JVMState, NewJavaValue, WasException};
use crate::better_java_stack::frames::PushableFrame;
use crate::class_loading::{check_initing_or_inited_class, ClassIntrinsicsData};
use crate::interpreter::common::ldc::load_class_constant_by_type;
use crate::interpreter_util::{new_object, run_constructor};
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
use crate::new_java_values::NewJavaValueHandle;
use crate::new_java_values::owned_casts::OwnedCastAble;
use crate::stdlib::java::lang::class_loader::ClassLoader;
use crate::stdlib::java::lang::string::JString;
use crate::stdlib::java::NewAsObjectOrJavaValue;
use crate::utils::run_static_or_virtual;

pub struct JClass<'gc> {
    pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
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

    pub fn get_class_loader<'l>(&self, _jvm: &'gc JVMState<'gc>, _int_state: &mut impl PushableFrame<'gc>) -> Result<Option<ClassLoader<'gc>>, WasException<'gc>> {
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
        // let class_class = jvm.classes.read().unwrap().class_class.clone();
        // let wrapped_id = self.normal_object.get_var_hidden(jvm, &class_class, HiddenJVMField::class_cpdtype_id_of_wrapped_in_array()).unwrap_int();
        // let not_wrapped_id = self.normal_object.get_var_hidden(jvm, &class_class, HiddenJVMField::class_cpdtype_id()).unwrap_int();
        // assert_ne!(wrapped_id, not_wrapped_id);
    }

    pub fn from_type<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, ptype: CPDType) -> Result<JClass<'gc>, WasException<'gc>> {
        let res = load_class_constant_by_type(jvm, int_state, ptype)?;
        Ok(res.cast_class().unwrap())//todo we should be able to safely turn handles that live for gc life without reentrant register
    }

    pub fn get_name<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<JString<'gc>, WasException<'gc>> {
        let class_class = check_initing_or_inited_class(jvm, int_state, CClassName::class().into()).unwrap();
        let res = run_static_or_virtual(jvm, int_state, &class_class, MethodName::method_getName(), &CMethodDescriptor::empty_args(CClassName::string().into()), vec![self.new_java_value()])?;
        Ok(res.expect("classes are known to have non-null names").cast_string_maybe_null().expect("classes are known to have non-null names"))
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
