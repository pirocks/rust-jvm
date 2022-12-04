use std::mem::size_of;
use std::ops::Deref;

use better_nonnull::BetterNonNull;
use runtime_class_stuff::FieldNumberAndFieldType;
use runtime_class_stuff::field_numbers::FieldNameAndClass;
use runtime_class_stuff::object_layout::FieldAccessor;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_descriptors::{CFieldDescriptor, CompressedFieldDescriptor};
use rust_jvm_common::compressed_classfile::compressed_types::CPDType;
use rust_jvm_common::compressed_classfile::field_names::FieldName;
use rust_jvm_common::runtime_type::RuntimeType;
use stage0::compiler::fields::recursively_find_field_number_and_type;

use crate::{check_initing_or_inited_class, JVMState, NewJavaValueHandle, WasException};
use crate::accessor_ext::AccessorExt;
use crate::better_java_stack::frames::HasFrame;
use crate::class_loading::assert_inited_or_initing_class;
use crate::interpreter::PostInstructionAction;
use crate::interpreter::real_interpreter_state::{InterpreterFrame, InterpreterJavaValue, RealInterpreterStateGuard};
use crate::static_vars::static_vars;

//
pub fn putstatic<'gc, 'k, 'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>, field_class_name: CClassName, field_name: FieldName, field_descriptor: &CFieldDescriptor) -> PostInstructionAction<'gc> {
    let target_classfile = match check_initing_or_inited_class(jvm, int_state.inner(), field_class_name.clone().into()) {
        Ok(target_classfile) => target_classfile,
        Err(WasException { exception_obj }) => {
            return PostInstructionAction::Exception { exception: WasException { exception_obj } };
        }
    };
    let mut entry_mut = int_state.current_frame_mut();
    let field_value = entry_mut.pop(field_descriptor.0.to_runtime_type().unwrap());
    static_vars(target_classfile.deref(), jvm).set(field_name, field_value.to_new_java_handle(jvm));
    PostInstructionAction::Next {}
}

pub fn putfield<'gc, 'k, 'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>, class_name: CClassName, field_name: FieldName, field_descriptor: &CFieldDescriptor) -> PostInstructionAction<'gc> {
    let mut entry_mut = int_state.current_frame_mut();
    let CompressedFieldDescriptor(field_type) = field_descriptor;
    let target_class = assert_inited_or_initing_class(jvm, class_name.clone().into());
    let FieldNumberAndFieldType { number, cpdtype } = recursively_find_field_number_and_type(target_class.unwrap_class_class(), FieldNameAndClass { field_name, class_name });
    //todo use regular object layout field accessors
    let val = entry_mut.pop(field_type.to_runtime_type().unwrap());
    let object_ref = entry_mut.pop(RuntimeType::object());
    match object_ref {
        InterpreterJavaValue::Object(o) => {
            unsafe {
                match o {
                    Some(x) => {
                        // if field_name.0.to_str(&jvm.string_pool) == "value" && field_type == &CompressedParsedDescriptorType::ShortType{
                        //     dbg!(x.as_ptr());
                        //     dbg!(val.to_raw());
                        // }
                        let field_pointer = BetterNonNull::from(x).offset((number.0 as usize * size_of::<u64>()) as isize).unwrap();
                        FieldAccessor::new(field_pointer.0, cpdtype).write_interpreter_jv(val, field_descriptor.0)
                        // let raw_field_ptr = x.as_ptr().add(field_number.0 as usize * size_of::<jlong>()) as *mut u64;
                        // assert_ne!(val.to_raw(), 0xDDDDDDDDDDDDDDDD);
                        // raw_field_ptr.write(val.to_raw());
                    }
                    None => {
                        todo!()/*return throw_npe(jvm, int_state);*/
                    }
                }
            }
        }
        _ => {
            dbg!(object_ref);
            panic!()
        }
    };
    PostInstructionAction::Next {}
}

//
pub fn getstatic<'gc, 'k, 'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>, field_class_name: CClassName, field_name: FieldName, field_descriptor: &CFieldDescriptor) -> PostInstructionAction<'gc> {  //todo make sure class pointer is updated correctly
    let field_value = match get_static_impl(jvm, int_state, field_class_name, field_name, field_descriptor.0) {
        Ok(val) => val,
        Err(WasException { exception_obj }) => return PostInstructionAction::Exception { exception: WasException { exception_obj } },
    };
    int_state.current_frame_mut().push(field_value.to_interpreter_jv());
    PostInstructionAction::Next {}
}

fn get_static_impl<'gc, 'k, 'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>, field_class_name: CClassName, field_name: FieldName, cpdtype: CPDType) -> Result<NewJavaValueHandle<'gc>, WasException<'gc>> {
    let target_class = check_initing_or_inited_class(jvm, int_state.inner(), field_class_name.clone().into())?;
    let temp = static_vars(target_class.deref(), jvm);
    Ok(temp.get(field_name, cpdtype))
}

pub fn getfield<'gc, 'k, 'l, 'j>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, class_name: CClassName, field_name: FieldName, field_desc: &CompressedFieldDescriptor) -> PostInstructionAction<'gc> {
    let target_class = assert_inited_or_initing_class(jvm, class_name.clone().into());
    let FieldNumberAndFieldType { number, cpdtype } = recursively_find_field_number_and_type(target_class.unwrap_class_class(), FieldNameAndClass { field_name, class_name });
    let object_ref = current_frame.pop(RuntimeType::object());
    unsafe {
        match object_ref {
            InterpreterJavaValue::Object(Some(x)) => {
                let field_pointer = BetterNonNull::from(x).offset((number.0 as usize * size_of::<u64>()) as isize).unwrap();
                let res = FieldAccessor::new(field_pointer.0, cpdtype).read_interpreter_jv(field_desc.0);
                current_frame.push(res);
                PostInstructionAction::Next {}
            }
            _ => {
                current_frame.inner().inner().debug_print_stack_trace(jvm);
                dbg!(object_ref);
                panic!()
            }
        }
    }
}
