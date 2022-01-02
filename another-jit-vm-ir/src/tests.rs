use std::mem::transmute;
use std::sync::RwLock;

use another_jit_vm::{Register, VMExitAction, VMState};

use crate::{IRInstr, IRVMExitType, IRVMState, IRVMStateInner, OwnedIRStack, RuntimeVMExitInput};
use crate::ir_stack::FRAME_HEADER_END_OFFSET;


#[test]
fn basic_ir_vm_exit() {
    let ir_vm_state: IRVMState<'_,()> = IRVMState {
        native_vm: VMState::new(),
        inner: RwLock::new(IRVMStateInner::new()),
    };
    let mut owned_ir_stack: OwnedIRStack = OwnedIRStack::new();
    let ir_vm_ref: &'_ IRVMState<'_,()> = unsafe { transmute(&ir_vm_state) };
    let frame_size = 0;
    let instructions = vec![IRInstr::VMExit2 { exit_type: IRVMExitType::TopLevelReturn }];
    let (ir_method_id, restart_points) = ir_vm_ref.add_function(instructions, frame_size, box |event, extra|{
        match event.exit_type {
            RuntimeVMExitInput::TopLevelReturn { .. } => {}
            _ => panic!()
        }
        VMExitAction::ExitVMCompletely { return_data: 0 }
    });
    let frame_pointer = owned_ir_stack.mmaped_top;
    ir_vm_state.run_method(ir_method_id,&mut owned_ir_stack,(),frame_pointer, frame_pointer);
}


#[test]
fn basic_ir_function_call() {
    let ir_vm_state: IRVMState<'_,()> = IRVMState {
        native_vm: VMState::new(),
        inner: RwLock::new(IRVMStateInner::new()),
    };
    let mut owned_ir_stack: OwnedIRStack = OwnedIRStack::new();
    let ir_vm_ref: &'_ IRVMState<'_,()> = unsafe { transmute(&ir_vm_state) };
    let frame_size = FRAME_HEADER_END_OFFSET;
    let to_call_function_instructions = vec![IRInstr::Return {
        return_val: None,
        temp_register_1: Register(1),
        temp_register_2: Register(2),
        temp_register_3: Register(3),
        temp_register_4: Register(4),
        frame_size
    }];
    let (to_call_ir_method_id, restart_points)= ir_vm_ref.add_function(to_call_function_instructions,0,box |event, extra|{
        todo!()
    });
    let to_call_function_pointer = ir_vm_ref.lookup_ir_method_id_pointer(to_call_ir_method_id);

    let instructions = vec![IRInstr::IRCall {
        temp_register_1: Register(1),
        temp_register_2: Register(2),
        current_frame_size: 0,
        new_frame_size: frame_size,
        target_address: to_call_function_pointer
    }, IRInstr::VMExit2 { exit_type: IRVMExitType::TopLevelReturn }];
    let (ir_method_id, restart_points) = ir_vm_ref.add_function(instructions, frame_size, box |event, extra|{
        match event.exit_type {
            RuntimeVMExitInput::TopLevelReturn { .. } => {}
            _ => panic!()
        }
        VMExitAction::ExitVMCompletely { return_data: 0 }
    });
    let frame_pointer = owned_ir_stack.mmaped_top;
    ir_vm_state.run_method(ir_method_id,&mut owned_ir_stack,(),frame_pointer, frame_pointer);
}
