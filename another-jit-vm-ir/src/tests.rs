use std::cell::RefCell;
use std::lazy::OnceCell;
use std::mem::transmute;
use std::ops::Deref;
use std::ptr::null_mut;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use libc::strcasecmp;

use another_jit_vm::{Register, VMExitAction};
use another_jit_vm::stack::OwnedNativeStack;
use rust_jvm_common::MethodId;
use rust_jvm_common::ptype::PType::Ref;

use crate::{IRInstr, IRInstructIndex, IRMethodID, IRStack, IRStackMut, IRVMExitAction, IRVMExitEvent, IRVMExitType, IRVMState, RestartPointID, RuntimeVMExitInput};
use crate::compiler::RestartPointGenerator;
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
        IRVMExitAction::ExitVMCompletely { return_data: 0 }
    });
    ir_stack.push_frame(todo!(), todo!(), todo!(), todo!(), &[]);
    ir_vm_ref.run_method(ir_method_id, ir_stack.current_frame_mut(), &mut ());
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
        IRVMExitAction::ExitVMCompletely { return_data: 0 }
    });
    ir_stack.push_frame(todo!(), todo!(), todo!(), todo!(), &[]);
    ir_vm_ref.run_method(ir_method_id, ir_stack.current_frame_mut(), &mut ());
    assert!(*was_exited.lock().unwrap().deref());
}


#[test]
fn idk_what_this_test_even_does() {
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
    let mut called_once = RefCell::new(false);
    let (to_call_ir_method_id, restart_points) = ir_vm_ref.add_function(to_call_function_instructions, 0, box move |ir_vm_exit, mut owned_ir_stack, self_, extra| {
        let target_method_id = calling_function_clone.get().cloned().unwrap();
        if !*called_once.borrow() {
            *called_once.borrow_mut() = true;
            owned_ir_stack.push_frame(todo!(), todo!(), todo!(), todo!(), &[]);
            let method_run_result = self_.run_method(target_method_id, owned_ir_stack.current_frame_mut(), &mut ());
            IRVMExitAction::ExitVMCompletely { return_data: method_run_result }
        } else {
            IRVMExitAction::ExitVMCompletely { return_data: 0 }
        }
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
    ir_stack.push_frame(todo!(), todo!(), todo!(), todo!(), &[]);
    ir_vm_ref.run_method(ir_method_id, ir_stack.current_frame_mut(), &mut ());
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
    let mut ir_stack: IRStack = IRStack {
        native: &mut owned_native_stack
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

    let (ir_method_to_call_second, restart_points) = ir_vm_ref.add_function(to_call_function_instructions_second, frame_size, box move |ir_vm_exit, owned_ir_stack, self_, extra| {
        eprintln!("Took exit on second call");
        owned_ir_stack.debug_print_stack_strace(self_);
        let ir_vm_exit: &IRVMExitEvent = ir_vm_exit;
        assert!(matches!(ir_vm_exit.exit_type,RuntimeVMExitInput::TopLevelReturn {..}));
        IRVMExitAction::RestartAtIndex { index: IRInstructIndex(1) }
    });
    assert_eq!(restart_points.iter().next().unwrap().1.0, 1);


    let to_call_function_instructions_first = vec![IRInstr::VMExit2 { exit_type: IRVMExitType::TopLevelReturn }, IRInstr::Return {
        return_val: None,
        temp_register_1: Register(1),
        temp_register_2: Register(2),
        temp_register_3: Register(3),
        temp_register_4: Register(4),
        frame_size,
    }];
    let functions_clone = functions.clone();
    let (ir_method_to_call_first, _) = ir_vm_ref.add_function(to_call_function_instructions_first, frame_size, box move |ir_vm_exit, owned_ir_stack, self_, extra| {
        eprintln!("Took exit on first call");
        let functions = functions_clone.borrow().unwrap();
        let self_: &'_ IRVMState<'_, ()> = self_;
        let mut owned_ir_stack: IRStackMut = owned_ir_stack;
        owned_ir_stack.push_frame(null_mut(), null_mut(), Some(functions.ir_method_calling_second), None, &[]);
        let current_frame_mut = owned_ir_stack.current_frame_mut();
        dbg!(current_frame_mut.downgrade().ir_method_id());
        self_.run_method(functions.ir_method_calling_second, current_frame_mut, extra);
        IRVMExitAction::RestartAtIndex { index: IRInstructIndex(1) }
    });

    let calling_function_first_instructions = vec![IRInstr::IRCall {
        temp_register_1: Register(1),
        temp_register_2: Register(2),
        current_frame_size: 0,
        new_frame_size: frame_size,
        target_address: ir_vm_ref.lookup_ir_method_id_pointer(ir_method_to_call_first),
    }, IRInstr::VMExit2 { exit_type: IRVMExitType::TopLevelReturn }];

    let (ir_method_calling_first, _) = ir_vm_ref.add_function(calling_function_first_instructions, frame_size, box move |ir_vm_exit, owned_ir_stack, self_, extra| {
        todo!()
    });


    let calling_function_second_instructions = vec![IRInstr::IRCall {
        temp_register_1: Register(1),
        temp_register_2: Register(2),
        current_frame_size: 0,
        new_frame_size: frame_size,
        target_address: ir_vm_ref.lookup_ir_method_id_pointer(ir_method_to_call_second),
    }, IRInstr::VMExit2 { exit_type: IRVMExitType::TopLevelReturn }];

    let (ir_method_calling_second, _) = ir_vm_ref.add_function(calling_function_second_instructions, frame_size, box move |ir_vm_exit, owned_ir_stack, self_, extra| {
        todo!()
    });


    *functions.borrow_mut() = Some(Functions {
        ir_method_to_call_second,
        ir_method_to_call_first,
        ir_method_calling_first,
        ir_method_calling_second,
    });

    ir_stack.push_frame(null_mut(), null_mut(), Some(ir_method_calling_first), None, &[]);
    ir_vm_ref.run_method(ir_method_calling_first, ir_stack.current_frame_mut(), &mut ());
}

