use std::mem::size_of;
use std::ptr::NonNull;

use itertools::Itertools;
use libc::c_void;

use another_jit_vm_ir::ir_stack::{IRFrameMut, IRFrameRef, OwnedIRStack};
use gc_memory_layout_common::layout::FRAME_HEADER_END_OFFSET;
use rust_jvm_common::loading::LoaderName;
use rust_jvm_common::NativeJavaValue;
use rust_jvm_common::runtime_type::RuntimeType;

use crate::{AllocatedHandle, JavaValueCommon, JVMState, MethodResolverImpl, NewJavaValue, NewJavaValueHandle, StackEntryPush};
use crate::interpreter::real_interpreter_state::InterpreterJavaValue;
use crate::interpreter_state::{NativeFrameInfo, OpaqueFrameInfo};
use crate::ir_to_java_layer::java_stack::OpaqueFrameIdOrMethodID;
use crate::java_values::native_to_new_java_value_rtype;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct FramePointer(pub NonNull<c_void>);

impl FramePointer {
    pub fn as_ptr(&self) -> *mut c_void {
        self.0.as_ptr()
    }

    pub fn as_const_ptr(&self) -> *const c_void {
        self.0.as_ptr() as *const c_void
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct StackDepth(u16);

//needs to keep track of operand stack for interpreter
//needs to have same underlying for interpreter and not-interpreter
//      follows that there needs to be a mechanism for non-interpreter frames in exits to know
//      operand stack depth
//needs to be fast
// one per java thread, needs to be
// maybe built on top of ir stack
pub struct JavaStack<'gc> {
    jvm: &'gc JVMState<'gc>,
    owned_ir_stack: OwnedIRStack,
    interpreter_frame_operand_stack_depths: Vec<(FramePointer, StackDepth)>,
    // current_frame_pointers: Vec<*mut c_void>,
    //frame pointer and depth
    // vm_exit_stack_depth: Option<StackDepth>,
    throw: Option<AllocatedHandle<'gc>>,//todo this should probably be in some kind of thread state thing
}

impl<'gc> JavaStack<'gc> {
    pub fn new_interpreter_frame<'l>(&'l mut self, frame_pointer: FramePointer) -> JavaInterpreterFrame<'gc, 'l> {
        JavaInterpreterFrame::from_frame_pointer_interpreter(self, frame_pointer)
    }

    pub fn exit_frame<'l>(&'l mut self, frame_pointer: FramePointer, stack_depth: Option<StackDepth>) -> JavaExitFrame<'gc, 'l> {
        JavaExitFrame { java_stack: self, frame_pointer, num_locals: todo!(), max_stack: todo!(), stack_depth }
    }
}

//need enter and exit native functions, enter taking an operand stack depth?

pub struct JavaExitFrame<'gc, 'l> {
    java_stack: &'l mut JavaStack<'gc>,
    frame_pointer: FramePointer,
    num_locals: u16,
    max_stack: u16,
    stack_depth: Option<StackDepth>,
    //get/set/etc
}

pub trait HasFrame<'gc> {
    fn frame_ref(&self) -> IRFrameRef;
    fn frame_mut(&mut self) -> IRFrameMut;
    fn jvm(&self) -> &'gc JVMState<'gc>;
    fn num_locals(&self) -> u16;
    fn max_stack(&self) -> u16;
    fn local_get(&self, i: u16, expected_type: RuntimeType) -> NewJavaValueHandle<'gc> {
        assert!(i < self.num_locals());
        let jvm = self.jvm();
        let ir_frame_ref = self.frame_ref();
        let data = ir_frame_ref.data(i as usize);//todo replace this with a layout lookup thing again
        let native_jv = NativeJavaValue { as_u64: data };
        native_to_new_java_value_rtype(native_jv, expected_type, jvm)
    }

    fn local_set(&mut self, i: u16, njv: NewJavaValue<'gc, '_>) {
        assert!(i < self.num_locals());
        let native_jv = njv.to_native();
        let ir_frame_mut = self.frame_mut();
        ir_frame_mut.write_data(i as usize, unsafe { native_jv.as_u64 });
    }

    fn os_set_from_start(&mut self, from_start: u16, njv: NewJavaValue<'gc, '_>) {
        let native_jv = njv.to_native();
        self.os_set_from_start_raw(from_start, unsafe { native_jv.as_u64 })
    }

    fn os_set_from_start_raw(&mut self, from_start: u16, raw: u64) {
        assert!(from_start < self.max_stack());
        let num_locals = self.num_locals() as usize;
        let ir_frame_mut = self.frame_mut();
        ir_frame_mut.write_data(num_locals + from_start as usize, raw);
    }

    fn os_get_from_start(&mut self, from_start: u16, expected_type: RuntimeType) -> NewJavaValueHandle<'gc> {
        assert!(from_start < self.max_stack());
        let ir_frame_ref = self.frame_ref();
        let num_locals = self.num_locals() as usize;
        let data = ir_frame_ref.data(num_locals + from_start as usize);//todo replace this with a layout lookup thing again
        let native_jv = NativeJavaValue { as_u64: data };
        native_to_new_java_value_rtype(native_jv, expected_type, self.jvm())
    }
}

impl<'gc, 'l> HasFrame<'gc> for JavaExitFrame<'gc, 'l> {
    fn frame_ref(&self) -> IRFrameRef {
        IRFrameRef {
            ptr: self.frame_pointer.as_ptr(),
            _ir_stack: &self.java_stack.owned_ir_stack,
        }
    }

    fn frame_mut(&mut self) -> IRFrameMut {
        IRFrameMut {
            ptr: self.frame_pointer.as_ptr(),
            ir_stack: &mut self.java_stack.owned_ir_stack,
        }
    }

    fn jvm(&self) -> &'gc JVMState<'gc> {
        self.java_stack.jvm
    }

    fn num_locals(&self) -> u16 {
        self.num_locals
    }

    fn max_stack(&self) -> u16 {
        self.max_stack
    }
}


impl<'gc, 'l> JavaExitFrame<'gc, 'l> {}

pub struct JavaInterpreterFrame<'gc, 'l> {
    java_stack: &'l mut JavaStack<'gc>,
    frame_ptr: FramePointer,
    num_locals: u16,
    max_stack: u16,
    current_operand_stack_depth: u16,
    //push, pop etc
}

impl<'gc, 'l> HasFrame<'gc> for JavaInterpreterFrame<'gc, 'l> {
    fn frame_ref(&self) -> IRFrameRef {
        IRFrameRef {
            ptr: self.frame_ptr.as_ptr(),
            _ir_stack: &self.java_stack.owned_ir_stack,
        }
    }

    fn frame_mut(&mut self) -> IRFrameMut {
        IRFrameMut {
            ptr: self.frame_ptr.as_ptr(),
            ir_stack: &mut self.java_stack.owned_ir_stack,
        }
    }

    fn jvm(&self) -> &'gc JVMState<'gc> {
        self.java_stack.jvm
    }

    fn num_locals(&self) -> u16 {
        self.num_locals
    }

    fn max_stack(&self) -> u16 {
        self.max_stack
    }
}

impl<'gc, 'l> JavaInterpreterFrame<'gc, 'l> {
    pub fn pop_frame(self) {}

    fn next_frame_pointer(&self) -> FramePointer {
        //todo need a better way of providing layout
        unsafe { FramePointer(NonNull::new(self.frame_ptr.as_ptr().sub(FRAME_HEADER_END_OFFSET + size_of::<u64>() * (self.num_locals as usize + self.max_stack as usize))).unwrap()) }
    }

    pub fn from_frame_pointer_interpreter(java_stack: &'l mut JavaStack<'gc>, frame_pointer: FramePointer) -> Self {
        let mut res = Self {
            java_stack,
            frame_ptr: frame_pointer,
            num_locals: 0,
            max_stack: 0,
            current_operand_stack_depth: 0,
        };
        let method_id = res.frame_ref().method_id().unwrap();
        let jvm = res.jvm();
        let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
        let view = rc.view();
        let method_view = view.method_view_i(method_i);
        let code = method_view.code_attribute().unwrap();
        res.num_locals = code.max_locals;
        res.max_stack = code.max_stack;
        res
    }

    fn get_top_level_exit_ptr(&self) -> NonNull<c_void> {
        let ir_vm_state = &self.jvm().java_vm_state.ir;
        let top_level_ir_method_id = ir_vm_state.get_top_level_return_ir_method_id();
        ir_vm_state.lookup_ir_method_id_pointer(top_level_ir_method_id)
    }

    pub fn push_frame<'k>(&'k mut self, stack_entry: StackEntryPush) -> JavaInterpreterFrame<'gc, 'k> {
        let current_frame_pointer = self.frame_ptr;
        let next_frame_pointer = self.next_frame_pointer();
        let top_level_exit_ptr = self.get_top_level_exit_ptr();
        let jvm = self.jvm();
        match stack_entry {
            StackEntryPush::Java { operand_stack, local_vars, method_id } => {
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
                    self.java_stack.owned_ir_stack.write_frame(
                        next_frame_pointer.as_ptr(),
                        top_level_exit_ptr.as_ptr(),
                        current_frame_pointer.as_ptr(),
                        ir_method_id,
                        wrapped_method_id.to_native(),
                        data.as_slice()
                    );
                }
                let (rc, method_i)= jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
                let view = rc.view();
                let method_view = view.method_view_i(method_i);
                let code = method_view.code_attribute().unwrap();
                JavaInterpreterFrame {
                    java_stack: &mut self.java_stack,
                    frame_ptr: next_frame_pointer,
                    num_locals: code.max_locals,
                    max_stack: code.max_stack,
                    current_operand_stack_depth: 0,
                }
            }
            StackEntryPush::Native { method_id, native_local_refs, local_vars, operand_stack } => {
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
                    self.java_stack.owned_ir_stack.write_frame(
                        next_frame_pointer.as_ptr(),
                        top_level_exit_ptr.as_ptr(),
                        current_frame_pointer.as_ptr(),
                        Some(ir_method_id),
                        wrapped_method_id.to_native(),
                        data.as_slice()
                    );
                }
                panic!()
            }
            StackEntryPush::Opaque { opaque_id, native_local_refs } => {
                let wrapped_opaque_id = OpaqueFrameIdOrMethodID::Opaque { opaque_id };
                let opaque_frame_info = OpaqueFrameInfo { native_local_refs, operand_stack: vec![] };
                let raw_frame_info_pointer = Box::into_raw(box opaque_frame_info);
                let data = [raw_frame_info_pointer as *const c_void as usize as u64];
                unsafe {
                    self.java_stack.owned_ir_stack.write_frame(
                        next_frame_pointer.as_ptr(),
                        top_level_exit_ptr.as_ptr(),
                        current_frame_pointer.as_ptr(),
                        None,
                        wrapped_opaque_id.to_native(),
                        data.as_slice()
                    );
                }
                panic!()
            }
        }
    }

    pub fn push_os(&mut self, njv: InterpreterJavaValue) {
        let current_depth = self.current_operand_stack_depth;
        self.os_set_from_start_raw(current_depth, njv.to_raw());
        self.current_operand_stack_depth += 1;
    }

    pub fn pop_os(&mut self, expected_type: RuntimeType) -> InterpreterJavaValue {
        if self.current_operand_stack_depth == 0 {
            panic!()
        }
        self.current_operand_stack_depth -= 1;
        let current_depth = self.current_operand_stack_depth;
        self.os_get_from_start(current_depth, expected_type).to_interpreter_jv()
    }
}

// don't have the function call vec thing
