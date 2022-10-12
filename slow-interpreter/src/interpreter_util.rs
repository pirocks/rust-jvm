use std::sync::Arc;

use classfile_view::view::HasAccessFlags;
use runtime_class_stuff::RuntimeClass;
use rust_jvm_common::compressed_classfile::CMethodDescriptor;
use rust_jvm_common::compressed_classfile::method_names::MethodName;

use crate::{AllocatedHandle, JavaValueCommon, JVMState, NewJavaValue, WasException};
use crate::better_java_stack::frames::PushableFrame;
use crate::class_loading::check_initing_or_inited_class;
use crate::interpreter::common::invoke::special::invoke_special_impl;
use crate::java_values::{default_value, JavaValue};
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;

//todo jni should really live in interpreter state

pub fn new_object_full<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, runtime_class: &'_ Arc<RuntimeClass<'gc>>) -> AllocatedHandle<'gc> {
    AllocatedHandle::NormalObject(new_object(jvm, int_state, runtime_class, false))
}

pub fn new_object<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, runtime_class: &'_ Arc<RuntimeClass<'gc>>, will_apply_intrinsic_data: bool) -> AllocatedNormalObjectHandle<'gc> {
    check_initing_or_inited_class(jvm, int_state, runtime_class.cpdtype()).expect("todo");
    let object_handle = JavaValue::new_object(jvm, runtime_class.clone(), will_apply_intrinsic_data);
    let _loader = jvm.classes.read().unwrap().get_initiating_loader(runtime_class);
    default_init_fields(jvm, &runtime_class, &object_handle);
    object_handle
}

fn default_init_fields<'gc, 'k>(jvm: &'gc JVMState<'gc>, current_class_pointer: &Arc<RuntimeClass<'gc>>, object_pointer: &'k AllocatedNormalObjectHandle<'gc>) {
    if let Some(super_) = current_class_pointer.unwrap_class_class().parent.as_ref() {
        default_init_fields(jvm, super_, object_pointer);
    }
    for field in current_class_pointer.view().fields() {
        if !field.is_static() {
            //todo should I look for constant val attributes?
            /*let _value_i = match field.constant_value_attribute() {
                None => {}
                Some(_i) => _i,
            };*/
            let name = field.field_name();
            let type_ = field.field_type();
            let val = default_value(type_);

            object_pointer.set_var(current_class_pointer, field.field_name(), val.as_njv());
            // unsafe {
            // *object_pointer.fields.get(&name).unwrap().get().as_mut().unwrap() = val;
            // }
        }
    }
}

pub fn run_constructor<'gc, 'l, 'k>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, target_classfile: Arc<RuntimeClass<'gc>>, full_args: Vec<NewJavaValue<'gc, 'k>>, descriptor: &CMethodDescriptor) -> Result<(), WasException<'gc>> {
    let target_classfile_view = target_classfile.view();
    let method_view = target_classfile_view.lookup_method(MethodName::constructor_init(), descriptor).unwrap();
    let md = method_view.desc();
    let res = invoke_special_impl(jvm, int_state, md, method_view.method_i(), target_classfile.clone(), full_args)?;
    assert!(res.is_none());
    Ok(())
}