
// #[no_mangle]
// unsafe extern "system" fn Java_sun_misc_Unsafe_allocateMemory(env: *mut JNIEnv, the_unsafe: jobject, len: jlong) -> jlong {
//     let res: i64 = libc::malloc(len as usize) as i64;
//     res
// }

use another_jit_vm::{IRMethodID, Register};
use another_jit_vm::intrinsic_helpers::IntrinsicHelperType;
use another_jit_vm_ir::compiler::{IRInstr, Size};
use gc_memory_layout_common::frame_layout::NativeStackframeMemoryLayout;
use rust_jvm_common::MethodId;
use crate::compiler_common::MethodResolver;

pub fn unsafe_allocate_memory<'gc>(resolver: &impl MethodResolver<'gc>, layout: &NativeStackframeMemoryLayout, method_id: MethodId, ir_method_id: IRMethodID) -> Option<Vec<IRInstr>> {
    let res = Register(0);
    let arg = Register(1);
    Some(vec![
        IRInstr::IRStart {
            temp_register: Register(2),
            ir_method_id,
            method_id,
            frame_size: layout.full_frame_size(),
            num_locals: resolver.num_locals(method_id) as usize,
        },
        IRInstr::LoadFPRelative {
            from: layout.local_var_entry(1),
            to: arg,
            size: Size::pointer()
        },
        IRInstr::CallIntrinsicHelper {
            intrinsic_helper_type: IntrinsicHelperType::Malloc,
            integer_args: vec![arg],
            integer_res: Some(res),
            float_args: vec![],
            float_res: None,
            double_args: vec![],
            double_res: None
        },
        IRInstr::Return {
            return_val: Some(res),
            temp_register_1: Register(1),
            temp_register_2: Register(2),
            temp_register_3: Register(3),
            temp_register_4: Register(4),
            frame_size: layout.full_frame_size(),
        },
    ])
}

///#[no_mangle]
// unsafe extern "system" fn Java_sun_misc_Unsafe_freeMemory(env: *mut JNIEnv, the_unsafe: jobject, ptr: jlong) {
//     libc::free(transmute(ptr))
// }

pub fn unsafe_free_memory<'gc>(resolver: &impl MethodResolver<'gc>, layout: &NativeStackframeMemoryLayout, method_id: MethodId, ir_method_id: IRMethodID) -> Option<Vec<IRInstr>> {
    let arg = Register(1);
    Some(vec![
        IRInstr::IRStart {
            temp_register: Register(2),
            ir_method_id,
            method_id,
            frame_size: layout.full_frame_size(),
            num_locals: resolver.num_locals(method_id) as usize,
        },
        IRInstr::LoadFPRelative {
            from: layout.local_var_entry(1),
            to: arg,
            size: Size::pointer()
        },
        IRInstr::CallIntrinsicHelper {
            intrinsic_helper_type: IntrinsicHelperType::Free,
            integer_args: vec![arg],
            integer_res: None,
            float_args: vec![],
            float_res: None,
            double_args: vec![],
            double_res: None
        },
        IRInstr::Return {
            return_val: None,
            temp_register_1: Register(1),
            temp_register_2: Register(2),
            temp_register_3: Register(3),
            temp_register_4: Register(4),
            frame_size: layout.full_frame_size(),
        },
    ])
}