use classfile_view::view::constant_info_view::{ConstantInfoView, StringView};
use rust_jvm_common::compressed_classfile::compressed_types::{CPDType};


use crate::{AllocatedHandle, JVMState, WasException};
use crate::better_java_stack::frames::PushableFrame;
use crate::class_objects::get_or_create_class_object;
use crate::new_java_values::NewJavaValueHandle;
use crate::stdlib::java::lang::string::JString;
use crate::stdlib::java::NewAsObjectOrJavaValue;
use crate::string_intern::intern_safe;

pub fn load_class_constant_by_type<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, res_class_type: CPDType) -> Result<NewJavaValueHandle<'gc>, WasException<'gc>> {
    let object = get_or_create_class_object(jvm, res_class_type.clone(), int_state)?;
    Ok(NewJavaValueHandle::Object(AllocatedHandle::NormalObject(object)))
}

fn load_string_constant<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, s: &StringView) -> NewJavaValueHandle<'gc> {
    let res_string = s.string();
    let before_intern = JString::from_rust(jvm, int_state, res_string).expect("todo");
    let string = intern_safe(jvm, before_intern.full_object().into());
    string.new_java_value_handle()
}

pub fn from_constant_pool_entry<'gc, 'l>(c: &ConstantInfoView, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> NewJavaValueHandle<'gc> {
    match &c {
        ConstantInfoView::Integer(i) => NewJavaValueHandle::Int(i.int),
        ConstantInfoView::Float(f) => NewJavaValueHandle::Float(f.float),
        ConstantInfoView::Long(l) => NewJavaValueHandle::Long(l.long),
        ConstantInfoView::Double(d) => NewJavaValueHandle::Double(d.double),
        ConstantInfoView::String(s) => {
            let string_value = load_string_constant(jvm, int_state, s);
            intern_safe(jvm, string_value.unwrap_object_nonnull()).new_java_value_handle()
        }
        _ => panic!(),
    }
}