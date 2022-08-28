use std::ptr::NonNull;
use std::sync::{Arc};

use libc::c_void;

use another_jit_vm_ir::ir_stack::{OwnedIRStack};
use rust_jvm_common::{ByteCodeOffset};
use rust_jvm_common::loading::LoaderName;

use crate::{AllocatedHandle, JavaValueCommon, JVMState, MethodResolverImpl, StackEntryPush};
use crate::better_java_stack::exit_frame::JavaExitFrame;
use crate::better_java_stack::interpreter_frame::JavaInterpreterFrame;
use crate::better_java_stack::java_stack_guard::JavaStackGuard;
use crate::better_java_stack::thread_remote_read_mechanism::SignalAccessibleJavaStackData;
use crate::interpreter_state::{NativeFrameInfo, OpaqueFrameInfo};
use crate::ir_to_java_layer::java_stack::OpaqueFrameIdOrMethodID;
use crate::stack_entry::{JavaFramePush, NativeFramePush, OpaqueFramePush};

#[cfg(test)]
pub mod test;
pub mod thread_remote_read_mechanism;
pub mod frames;
pub mod interpreter_frame;
pub mod exit_frame;
pub mod remote_frame;
pub mod java_stack_guard;
pub mod opaque_frame;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Ord, PartialOrd)]
pub struct FramePointer(pub NonNull<c_void>);

impl FramePointer {
    pub fn as_ptr(&self) -> *mut c_void {
        self.0.as_ptr()
    }

    pub fn as_const_ptr(&self) -> *const c_void {
        self.0.as_ptr() as *const c_void
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct StackDepth(u16);

//needs to keep track of operand stack for interpreter
//      needs to be viewable by other threads
//needs to have same underlying for interpreter and not-interpreter
//      follows that there needs to be a mechanism for non-interpreter frames in exits to know
//      operand stack depth
//needs to be fast
// one per java thread, needs to be
// maybe built on top of ir stack
//todo needs to be interruptable and viewable once interrupted
// todo if in guest then can send stack pointer.
// need a in guest/not in guest atomic, per thread atomic.
pub struct JavaStack<'gc> {
    jvm: &'gc JVMState<'gc>,
    owned_ir_stack: OwnedIRStack,
    interpreter_frame_operand_stack_depths: Vec<(FramePointer, InterpreterFrameState)>,
    throw: Option<AllocatedHandle<'gc>>,
    //todo this should probably be in some kind of thread state thing
    thread_stack_data: Arc<SignalAccessibleJavaStackData>,
    has_been_used: bool
}

#[derive(Copy, Clone, Debug)]
pub struct InterpreterFrameState {
    stack_depth: StackDepth,
    current_pc: ByteCodeOffset,
}

impl<'gc> JavaStack<'gc> {
    pub fn new(jvm: &'gc JVMState<'gc>, owned_ir_stack: OwnedIRStack, thread_stack_data: Arc<SignalAccessibleJavaStackData>) -> Self {
        Self {
            jvm,
            owned_ir_stack,
            interpreter_frame_operand_stack_depths: vec![],
            throw: None,
            thread_stack_data,
            has_been_used: false
        }
    }

    pub fn assert_interpreter_frame_operand_stack_depths_sorted(&self) {
        assert!(self.interpreter_frame_operand_stack_depths.iter().map(|(frame_ptr, _)| *frame_ptr).is_sorted());
    }
}


//need enter and exit native functions, enter taking an operand stack depth?


fn get_top_level_exit_ptr<'gc>(jvm: &'gc JVMState<'gc>) -> NonNull<c_void> {
    let ir_vm_state = &jvm.java_vm_state.ir;
    let top_level_ir_method_id = ir_vm_state.get_top_level_return_ir_method_id();
    ir_vm_state.lookup_ir_method_id_pointer(top_level_ir_method_id)
}


fn push_java_frame<'gc, 'k>(java_stack_guard: &'k mut JavaStackGuard<'gc>,
                            current_frame_pointer: FramePointer,
                            next_frame_pointer: FramePointer,
java_stack_entry: JavaFramePush
) -> JavaInterpreterFrame<'gc, 'k>{
    let JavaFramePush { method_id, local_vars, operand_stack } = java_stack_entry;
    assert_eq!(jvm.num_local_var_slots(method_id) as usize, local_vars.len());
    let ir_method_id = jvm.java_vm_state.try_lookup_method_ir_method_id(method_id);
    let mut data = vec![];
    for local_var in local_vars {
        if let Some(Some(obj)) = local_var.try_unwrap_object_alloc() {
            jvm.gc.memory_region.lock().unwrap().find_object_allocated_type(obj.ptr());
        }
        data.push(unsafe { local_var.to_native().as_u64 });
    }
    for jv in operand_stack {
        data.push(unsafe { jv.to_native().as_u64 });
    }
    let wrapped_method_id = OpaqueFrameIdOrMethodID::Method { method_id: method_id as u64 };
    unsafe {
        java_stack_guard.guard.as_mut().unwrap().owned_ir_stack.write_frame(
            next_frame_pointer.0,
            top_level_exit_ptr.as_ptr(),
            current_frame_pointer.as_ptr(),
            ir_method_id,
            wrapped_method_id.to_native(),
            data.as_slice(),
        );
    }
    let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
    let view = rc.view();
    let method_view = view.method_view_i(method_i);
    let code = method_view.code_attribute().unwrap();
    JavaInterpreterFrame {
        java_stack: java_stack_guard,
        frame_ptr: next_frame_pointer,
        num_locals: code.max_locals,
        max_stack: code.max_stack,
        current_operand_stack_depth: 0,
    }
}

fn push_frame_native<'gc, 'k>(java_stack_guard: &'k mut JavaStackGuard<'gc>,
                              current_frame_pointer: FramePointer,
                              next_frame_pointer: FramePointer,
                              stack_entry: NativeFramePush){
    jvm.java_vm_state.add_method_if_needed(jvm, &MethodResolverImpl { jvm, loader: LoaderName::BootstrapLoader/*todo fix*/ }, method_id, false);
    let ir_method_id = jvm.java_vm_state.lookup_method_ir_method_id(method_id);
    let (rc, _) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
    let loader = jvm.classes.read().unwrap().get_initiating_loader(&rc);
    assert_eq!(jvm.num_local_vars_native(method_id) as usize, local_vars.len());
    let native_frame_info = NativeFrameInfo {
        method_id,
        loader,
        native_local_refs,
        // local_vars: local_vars.iter().map(|njv|njv.to_native()).collect(),
        operand_stack: operand_stack.iter().map(|njv| njv.to_native()).collect(),
    };
    let raw_frame_info_pointer = Box::into_raw(box native_frame_info);
    let wrapped_method_id = OpaqueFrameIdOrMethodID::Method { method_id: method_id as u64 };
    //todo use NativeStackframeMemoryLayout for this
    let mut data = local_vars.iter().map(|local_var| unsafe { local_var.to_native().as_u64 }).collect_vec();
    data.push(raw_frame_info_pointer as *const c_void as usize as u64);
    unsafe {
        java_stack_guard.guard.as_mut().unwrap().owned_ir_stack.write_frame(
            next_frame_pointer.0,
            top_level_exit_ptr.as_ptr(),
            current_frame_pointer.as_ptr(),
            Some(ir_method_id),
            wrapped_method_id.to_native(),
            data.as_slice(),
        );
    }
    panic!()
}

fn push_opaque_frame<'gc,'k>(java_stack_guard: &'k mut JavaStackGuard<'gc>, current_frame_pointer : FramePointer, next_frame_pointer: FramePointer, opaque_frame: OpaqueFramePush) {
    let wrapped_opaque_id = OpaqueFrameIdOrMethodID::Opaque { opaque_id };
    let opaque_frame_info = OpaqueFrameInfo { native_local_refs, operand_stack: vec![] };
    let raw_frame_info_pointer = Box::into_raw(box opaque_frame_info);
    let data = [raw_frame_info_pointer as *const c_void as usize as u64];
    unsafe {
        java_stack_guard.guard.as_mut().unwrap().owned_ir_stack.write_frame(
            next_frame_pointer.0,
            top_level_exit_ptr.as_ptr(),
            current_frame_pointer.as_ptr(),
            None,
            wrapped_opaque_id.to_native(),
            data.as_slice(),
        );
    }
    panic!()
}

// fn push_interpreter<'gc, 'k>(
//     java_stack_guard: &'k mut JavaStackGuard<'gc>,
//     current_frame_pointer: FramePointer,
//     next_frame_pointer: FramePointer,
//     stack_entry: StackEntryPush
// ) -> JavaInterpreterFrame<'gc, 'k> {
//     let jvm = java_stack_guard.jvm();
//     let top_level_exit_ptr = get_top_level_exit_ptr(jvm);
//     match stack_entry {
//         StackEntryPush::Java { operand_stack, local_vars, method_id } => {
//             todo!()
//         }
//         StackEntryPush::Native { method_id, native_local_refs, local_vars, operand_stack } => {
//
//         }
//         StackEntryPush::Opaque { opaque_id, native_local_refs } => {
//
//         }
//     }
// }

impl<'gc, 'k> JavaExitFrame<'gc, 'k> {}
