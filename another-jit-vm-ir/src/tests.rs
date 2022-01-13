use std::mem::transmute;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use another_jit_vm::{Register, VMExitAction};
use another_jit_vm::stack::OwnedNativeStack;

use crate::{IRInstr, IRVMExitType, IRVMState, IRStack, RuntimeVMExitInput};
use crate::ir_stack::FRAME_HEADER_END_OFFSET;

#[test]
fn basic_ir_vm_exit() {
    let was_exited = Arc::new(Mutex::new(false));
    let ir_vm_state: IRVMState<'_, ()> = IRVMState::new(box |ir_event, ir_stack_ref_mut, self_, _| {
        panic!()
    });
    let mut owned_native_stack = OwnedNativeStack::new();
    let mut ir_stack: IRStack = IRStack{
        native: &mut owned_native_stack
    };
    let ir_vm_ref: &'_ IRVMState<'_, ()> = unsafe { transmute(&ir_vm_state) };
    let frame_size = 0;
    let instructions = vec![IRInstr::VMExit2 { exit_type: IRVMExitType::TopLevelReturn }];
    let (ir_method_id, restart_points) = ir_vm_ref.add_function(instructions, frame_size, box |ir_event, ir_stack_ref_mut, self_, _| {
        match ir_event.exit_type {
            RuntimeVMExitInput::TopLevelReturn { .. } => {
                *was_exited.clone().lock().unwrap() = true;
            }
            _ => panic!()
        }
        VMExitAction::ExitVMCompletely { return_data: 0 }
    });
    let frame_pointer = ir_stack.native.mmaped_top;
    ir_vm_ref.run_method(ir_method_id, &mut ir_stack, (), frame_pointer, frame_pointer);
    assert!(*was_exited.lock().unwrap().deref());
}


#[test]
fn basic_ir_function_call() {
    let was_exited = Arc::new(Mutex::new(false));
    let ir_vm_state: IRVMState<'_, ()> = IRVMState::new(box |ir_event, ir_stack_ref_mut, self_, _| {
        panic!()
    });
    let mut owned_native_stack = OwnedNativeStack::new();
    let mut ir_stack: IRStack = IRStack{
        native: &mut owned_native_stack
    };
    let ir_vm_ref: &'_ IRVMState<'_, ()> = unsafe { transmute(&ir_vm_state) };
    let frame_size = FRAME_HEADER_END_OFFSET;
    let to_call_function_instructions = vec![IRInstr::Return {
        return_val: None,
        temp_register_1: Register(1),
        temp_register_2: Register(2),
        temp_register_3: Register(3),
        temp_register_4: Register(4),
        frame_size,
    }];
    let (to_call_ir_method_id, restart_points) = ir_vm_ref.add_function(to_call_function_instructions, 0, box |ir_event, owned_ir_stacl, self_, extra| {
        panic!()
    });
    let to_call_function_pointer = ir_vm_ref.lookup_ir_method_id_pointer(to_call_ir_method_id);

    let instructions = vec![IRInstr::IRCall {
        temp_register_1: Register(1),
        temp_register_2: Register(2),
        current_frame_size: 0,
        new_frame_size: frame_size,
        target_address: to_call_function_pointer,
    }, IRInstr::VMExit2 { exit_type: IRVMExitType::TopLevelReturn }];
    let (ir_method_id, restart_points) = ir_vm_ref.add_function(instructions, frame_size, box |ir_event, owned_ir_stacl, self_, extra| {
        match ir_event.exit_type {
            RuntimeVMExitInput::TopLevelReturn { .. } => {
                *was_exited.clone().lock().unwrap() = true;
            }
            _ => panic!()
        }
        VMExitAction::ExitVMCompletely { return_data: 0 }
    });
    let frame_pointer = ir_stack.native.mmaped_top;
    ir_vm_ref.run_method(ir_method_id, &mut ir_stack, (), frame_pointer, frame_pointer);
    assert!(*was_exited.lock().unwrap().deref());
}


#[test]
fn nested_basic_ir_function_call_on_same_stack() {
    let ir_vm_state: IRVMState<'_, ()> = IRVMState::new(box |event, stack, self_, extra| {
        match event.exit_type {
            RuntimeVMExitInput::TopLevelReturn { return_value } => {
                todo!()
                // previously_exited  = true;
                /*if stack.mmaped_top == event.exiting_frame_position_rbp{
                    todo!()
                }else {
                    //todo this is really messy go back to per method handlers?
                    if !previously_exited{
                        // self_.run_method()
                        todo!()
                    }
                    todo!()
                }*/
            }
            _ => panic!()
        }
    });
    let mut owned_native_stack = OwnedNativeStack::new();
    let mut ir_stack: IRStack = IRStack{
        native: &mut owned_native_stack
    };
    let ir_vm_ref: &'_ IRVMState<'_, ()> = unsafe { transmute(&ir_vm_state) };
    let frame_size = FRAME_HEADER_END_OFFSET;
    let to_call_function_instructions = vec![IRInstr::VMExit2 { exit_type: IRVMExitType::TopLevelReturn }, IRInstr::Return {
        return_val: None,
        temp_register_1: Register(1),
        temp_register_2: Register(2),
        temp_register_3: Register(3),
        temp_register_4: Register(4),
        frame_size,
    }];
    let (to_call_ir_method_id, restart_points) = ir_vm_ref.add_function(to_call_function_instructions, 0, box |ir_vm_exit, owned_ir_stack, self_, extra| {

        todo!()
    });
    let to_call_function_pointer = ir_vm_ref.lookup_ir_method_id_pointer(to_call_ir_method_id);

    let instructions = vec![IRInstr::IRCall {
        temp_register_1: Register(1),
        temp_register_2: Register(2),
        current_frame_size: 0,
        new_frame_size: frame_size,
        target_address: to_call_function_pointer,
    }, IRInstr::VMExit2 { exit_type: IRVMExitType::TopLevelReturn }];
    let (ir_method_id, restart_points) = ir_vm_ref.add_function(instructions, frame_size, box |ir_vm_exit, owned_ir_stacl, self_, extra| {
        todo!()
    });
    let frame_pointer = ir_stack.native.mmaped_top;
    ir_vm_ref.run_method(ir_method_id, &mut ir_stack, (), frame_pointer, frame_pointer);
}
