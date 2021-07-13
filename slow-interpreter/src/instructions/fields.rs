use rust_jvm_common::compressed_classfile::CompressedFieldDescriptor;
use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName};
use rust_jvm_common::runtime_type::RuntimeType;
use verification::verifier::instructions::special::extract_field_descriptor;

use crate::{InterpreterStateGuard, JVMState};
use crate::class_loading::{assert_inited_or_initing_class, check_initing_or_inited_class};
use crate::interpreter::WasException;
use crate::java_values::JavaValue;
use crate::utils::throw_npe;

pub fn putstatic(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, cp: u16) {
    let view = int_state.current_class_view(jvm);
    let (field_class_name, field_name, field_descriptor) = extract_field_descriptor(&jvm.string_pool, cp, &*view);
    let target_classfile = assert_inited_or_initing_class(jvm, field_class_name.clone().into());
    let mut entry_mut = int_state.current_frame_mut();
    let mut stack = entry_mut.operand_stack_mut();
    let field_value = stack.pop(Some(field_descriptor.0.to_runtime_type().unwrap())).unwrap();
    target_classfile.static_vars().insert(field_name, field_value);
}

pub fn putfield(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, cp: u16) {
    let view = int_state.current_class_view(jvm);
    let (field_class_name, field_name, CompressedFieldDescriptor(field_type)) = extract_field_descriptor(&jvm.string_pool, cp, &*view);
    let target_class = assert_inited_or_initing_class(jvm, field_class_name.clone().into());
    let mut entry_mut = int_state.current_frame_mut();
    let stack = &mut entry_mut.operand_stack_mut();
    let val = stack.pop(Some(field_type.to_runtime_type().unwrap())).unwrap();
    let object_ref = stack.pop(Some(RuntimeType::object())).unwrap();
    match object_ref {
        JavaValue::Object(o) => {
            {
                match o {
                    Some(x) => x,
                    None => {
                        return throw_npe(jvm, int_state);
                    }
                }.unwrap_normal_object().set_var(target_class, field_name, val, field_type);
            }
        }
        _ => {
            dbg!(object_ref);
            panic!()
        }
    }
}

pub fn get_static(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, cp: u16) {
    //todo make sure class pointer is updated correctly

    let view = int_state.current_class_view(jvm);
    let (field_class_name, field_name, _field_descriptor) = extract_field_descriptor(&jvm.string_pool, cp, &*view);
    let field_value = match match get_static_impl(jvm, int_state, field_class_name, field_name) {
        Ok(val) => val,
        Err(WasException {}) => return
    } {
        None => { return; }
        Some(val) => val
    };
    int_state.push_current_operand_stack(field_value);
}

fn get_static_impl(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, field_class_name: CClassName, field_name: FieldName) -> Result<Option<JavaValue<'gc_life>>, WasException> {
    let target_classfile = check_initing_or_inited_class(jvm, int_state, field_class_name.clone().into())?;
    //todo handle interfaces in setting as well
    for interfaces in target_classfile.view().interfaces() {
        let interface_lookup_res = get_static_impl(jvm, int_state, interfaces.interface_name(), field_name.clone())?;
        if interface_lookup_res.is_some() {
            return Ok(interface_lookup_res);
        }
    }
    let temp = target_classfile.static_vars();
    let attempted_get = temp.get(&field_name);
    let field_value = match attempted_get {
        None => {
            let possible_super = target_classfile.view().super_name();
            match possible_super {
                None => None,
                Some(super_) => { return get_static_impl(jvm, int_state, super_, field_name).into(); }
            }
        }
        Some(val) => {
            val.clone().into()
        }
    };
    Ok(field_value)
}

pub fn get_field(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, cp: u16, _debug: bool) {
    let current_frame = int_state.current_frame();
    let view = current_frame.class_pointer(jvm).view();
    let (field_class_name, field_name, CompressedFieldDescriptor(field_type)) = extract_field_descriptor(&jvm.string_pool, cp, &*view);
    let target_class_pointer = assert_inited_or_initing_class(jvm, field_class_name.into());
    let object_ref = int_state.current_frame_mut().pop(Some(RuntimeType::object()));
    match object_ref {
        JavaValue::Object(o) => {
            let res = o.unwrap().unwrap_normal_object().get_var(jvm, target_class_pointer, field_name, field_type);
            int_state.current_frame_mut().push(res);
        }
        _ => panic!(),
    }
}

