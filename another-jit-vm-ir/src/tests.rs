use std::lazy::OnceCell;
use std::mem::transmute;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use another_jit_vm::{Register, VMExitAction};
use another_jit_vm::stack::OwnedNativeStack;

use crate::{IRInstr, IRMethodID, IRStack, IRStackMut, IRVMExitType, IRVMState, RuntimeVMExitInput};
use crate::ir_stack::FRAME_HEADER_END_OFFSET;

#[test]
fn basic_ir_vm_exit() {
    let was_exited = Arc::new(Mutex::new(false));
    let ir_vm_state: IRVMState<'_, ()> = IRVMState::new();
    let mut owned_native_stack = OwnedNativeStack::new();
    let mut ir_stack: IRStack = IRStack {
        native: &mut owned_native_stack
    };
    let mut ir_stack = IRStackMut::from_stack_start(&mut ir_stack);
    let ir_vm_ref: &'_ IRVMState<'_, ()> = unsafe { transmute(&ir_vm_state) };
    let frame_size = 0;
    let instructions = vec![IRInstr::VMExit2 { exit_type: IRVMExitType::TopLevelReturn }];
    let was_exited_clone = was_exited.clone();
    let (ir_method_id, restart_points) = ir_vm_ref.add_function(instructions, frame_size, box move |ir_event, ir_stack_ref_mut, self_, _| {
        match ir_event.exit_type {
            RuntimeVMExitInput::TopLevelReturn { .. } => {
                *was_exited_clone.lock().unwrap() = true;
            }
            _ => panic!()
        }
        VMExitAction::ExitVMCompletely { return_data: 0 }
    });
    ir_vm_ref.run_method(ir_method_id, &mut ir_stack, ());
    assert!(*was_exited.lock().unwrap().deref());
}


#[test]
fn basic_ir_function_call() {
    let was_exited = Arc::new(Mutex::new(false));
    let ir_vm_state: IRVMState<'_, ()> = IRVMState::new();
    let mut owned_native_stack = OwnedNativeStack::new();
    let mut ir_stack: IRStack = IRStack {
        native: &mut owned_native_stack
    };
    let mut ir_stack = IRStackMut::from_stack_start(&mut ir_stack);
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
    let was_exited_clone = was_exited.clone();
    let (ir_method_id, restart_points) = ir_vm_ref.add_function(instructions, frame_size, box move |ir_event, owned_ir_stacl, self_, extra| {
        match ir_event.exit_type {
            RuntimeVMExitInput::TopLevelReturn { .. } => {
                *was_exited_clone.lock().unwrap() = true;
            }
            _ => panic!()
        }
        VMExitAction::ExitVMCompletely { return_data: 0 }
    });
    ir_vm_ref.run_method(ir_method_id, &mut ir_stack, ());
    assert!(*was_exited.lock().unwrap().deref());
}


#[test]
fn nested_basic_ir_function_call_on_same_stack() {
    let ir_vm_state: IRVMState<'_, ()> = IRVMState::new();
    let mut owned_native_stack = OwnedNativeStack::new();
    let mut ir_stack: IRStack = IRStack {
        native: &mut owned_native_stack
    };
    let mut ir_stack = IRStackMut::from_stack_start(&mut ir_stack);
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
    let calling_function: Rc<OnceCell<IRMethodID>> = Rc::new(OnceCell::new());
    let calling_function_clone = calling_function.clone();
    let (to_call_ir_method_id, restart_points) = ir_vm_ref.add_function(to_call_function_instructions, 0, box move |ir_vm_exit, mut owned_ir_stack, self_, extra| {
        let target_method_id = calling_function_clone.get().cloned().unwrap();
        self_.run_method(target_method_id, &mut owned_ir_stack, ());
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
    calling_function.set(ir_method_id).unwrap();
    ir_vm_ref.run_method(ir_method_id, &mut ir_stack, ());
}
