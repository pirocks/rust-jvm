use std::cell::RefCell;
use std::mem::transmute;
use std::ops::Deref;
use std::ptr::null_mut;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use another_jit_vm::Register;
use another_jit_vm::stack::OwnedNativeStack;

use crate::{IRInstr, IRInstructIndex, IRMethodID, IRStackMut, IRVMExitAction, IRVMExitEvent, IRVMExitType, IRVMState, RuntimeVMExitInput};
use crate::compiler::RestartPointGenerator;
use crate::ir_stack::{FRAME_HEADER_END_OFFSET, OwnedIRStack};

#[test]
fn basic_ir_vm_exit() {
    let was_exited = Arc::new(Mutex::new(false));
    let ir_vm_state: IRVMState<'_, ()> = IRVMState::new();
    let mut owned_native_stack = OwnedNativeStack::new();
    let mut ir_stack: OwnedIRStack = OwnedIRStack {
        native: owned_native_stack
    };
    let mut ir_stack = IRStackMut::from_stack_start(&mut ir_stack);
    let ir_vm_ref: &'_ IRVMState<'_, ()> = unsafe { transmute(&ir_vm_state) };
    let frame_size = FRAME_HEADER_END_OFFSET;
    let instructions = vec![IRInstr::VMExit2 { exit_type: IRVMExitType::TopLevelReturn }];
    let was_exited_clone = was_exited.clone();
    let (ir_method_id, restart_points) = ir_vm_ref.add_function(instructions, frame_size);
    let frame_guard = ir_stack.push_frame(null_mut(), Some(ir_method_id), -1, &[], ir_vm_ref);
    ir_vm_ref.run_method(ir_method_id, &mut ir_stack.current_frame_mut(), &mut ());
    ir_stack.pop_frame(frame_guard);
    assert!(*was_exited.lock().unwrap().deref());
}


#[test]
fn basic_ir_function_call() {
    let was_exited = Arc::new(Mutex::new(false));
    let ir_vm_state: IRVMState<'_, ()> = IRVMState::new();
    let mut owned_native_stack = OwnedNativeStack::new();
    let mut ir_stack: OwnedIRStack = OwnedIRStack {
        native: owned_native_stack
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
    let (to_call_ir_method_id, restart_points) = ir_vm_ref.add_function(to_call_function_instructions, frame_size);
    let to_call_function_pointer = ir_vm_ref.lookup_ir_method_id_pointer(to_call_ir_method_id);

    let instructions = vec![IRInstr::IRCall {
        temp_register_1: Register(1),
        temp_register_2: Register(2),
        current_frame_size: 0,
        new_frame_size: frame_size,
        target_address: to_call_function_pointer,
    }, IRInstr::VMExit2 { exit_type: IRVMExitType::TopLevelReturn }];
    let was_exited_clone = was_exited.clone();
    let (ir_method_id, restart_points) = ir_vm_ref.add_function(instructions, frame_size);
    let frame_guard = ir_stack.push_frame(null_mut(), Some(ir_method_id), -1, &[], ir_vm_ref);
    ir_vm_ref.run_method(ir_method_id, &mut ir_stack.current_frame_mut(), &mut ());
    ir_stack.pop_frame(frame_guard);
    assert!(*was_exited.lock().unwrap().deref());
}


#[test]
fn nested_basic_ir_function_call_on_same_stack() {
    #[derive(Copy, Clone)]
    struct Functions {
        ir_method_to_call_second: IRMethodID,
        ir_method_to_call_first: IRMethodID,
        ir_method_calling_first: IRMethodID,
        ir_method_calling_second: IRMethodID,
    }

    let functions: Rc<RefCell<Option<Functions>>> = Rc::new(RefCell::new(None));

    let ir_vm_state: IRVMState<'_, ()> = IRVMState::new();
    let mut owned_native_stack = OwnedNativeStack::new();
    let mut ir_stack: OwnedIRStack = OwnedIRStack {
        native: owned_native_stack
    };
    let mut ir_stack = IRStackMut::from_stack_start(&mut ir_stack);
    let ir_vm_ref: &'_ IRVMState<'_, ()> = unsafe { transmute(&ir_vm_state) };
    let frame_size = FRAME_HEADER_END_OFFSET;
    let mut restart_point_gen = RestartPointGenerator::new();
    let restart_point_0 = restart_point_gen.new_restart_point();
    let to_call_function_instructions_second = vec![IRInstr::VMExit2 { exit_type: IRVMExitType::TopLevelReturn }, IRInstr::RestartPoint(restart_point_0), IRInstr::Return {
        return_val: None,
        temp_register_1: Register(1),
        temp_register_2: Register(2),
        temp_register_3: Register(3),
        temp_register_4: Register(4),
        frame_size,
    }];

    let (ir_method_to_call_second, restart_points) = ir_vm_ref.add_function(to_call_function_instructions_second, frame_size);
    assert_eq!(restart_points.iter().next().unwrap().1.0, 1);


    let to_call_function_instructions_first = vec![IRInstr::VMExit2 { exit_type: IRVMExitType::TopLevelReturn }, IRInstr::Return {
        return_val: None,
        temp_register_1: Register(1),
        temp_register_2: Register(2),
        temp_register_3: Register(3),
        temp_register_4: Register(4),
        frame_size,
    }];
    const NESTED_IR_METHOD_EXPECTED_RES: u64 = 10;
    let functions_clone = functions.clone();
    let (ir_method_to_call_first, _) = ir_vm_ref.add_function(to_call_function_instructions_first, frame_size);

    let calling_function_first_instructions = vec![IRInstr::IRCall {
        temp_register_1: Register(1),
        temp_register_2: Register(2),
        current_frame_size: 0,
        new_frame_size: frame_size,
        target_address: ir_vm_ref.lookup_ir_method_id_pointer(ir_method_to_call_first),
    }, IRInstr::VMExit2 { exit_type: IRVMExitType::TopLevelReturn }];

    let (ir_method_calling_first, _) = ir_vm_ref.add_function(calling_function_first_instructions, frame_size);


    let functions_clone = functions.clone();
    let calling_function_second_instructions = vec![IRInstr::IRCall {
        temp_register_1: Register(1),
        temp_register_2: Register(2),
        current_frame_size: 0,
        new_frame_size: frame_size,
        target_address: ir_vm_ref.lookup_ir_method_id_pointer(ir_method_to_call_second),
    }, IRInstr::NOP, IRInstr::VMExit2 { exit_type: IRVMExitType::TopLevelReturn }];

    let (ir_method_calling_second, _) = ir_vm_ref.add_function(calling_function_second_instructions, frame_size);


    *functions.borrow_mut() = Some(Functions {
        ir_method_to_call_second,
        ir_method_to_call_first,
        ir_method_calling_first,
        ir_method_calling_second,
    });

    let frame_guard = ir_stack.push_frame(null_mut(), Some(ir_method_calling_first), -1, &[], ir_vm_ref);
    {
        let mut frame_mut = ir_stack.current_frame_mut();
        let res = ir_vm_ref.run_method(ir_method_calling_first, &mut frame_mut, &mut ());
        assert_eq!(res, 0);
    }
    ir_stack.pop_frame(frame_guard);
}

