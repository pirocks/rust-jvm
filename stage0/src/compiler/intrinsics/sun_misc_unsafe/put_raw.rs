use another_jit_vm::{IRMethodID, Register};
use another_jit_vm_ir::compiler::{IRInstr, Size};
use gc_memory_layout_common::layout::NativeStackframeMemoryLayout;
use rust_jvm_common::MethodId;
use crate::compiler::CompilerLabeler;
use crate::compiler_common::MethodResolver;

///#[no_mangle]
//         unsafe extern "system" fn Java_sun_misc_Unsafe_putLong__JJ(env: *mut JNIEnv, the_unsafe: jobject, ptr: jlong, val: jlong) {
//             let ptr: *mut i64 = transmute(ptr);
//             ptr.write(val);
//         }

pub fn unsafe_put_long<'gc>(resolver: &impl MethodResolver<'gc>, layout: &NativeStackframeMemoryLayout, _labeler: &mut CompilerLabeler, method_id: MethodId, ir_method_id: IRMethodID) -> Option<Vec<IRInstr>> {
    let ptr = Register(0);
    let val = Register(1);
    return Some(vec![
        IRInstr::IRStart {
            temp_register: Register(2),
            ir_method_id,
            method_id,
            frame_size: layout.full_frame_size(),
            num_locals: resolver.num_locals(method_id) as usize,
        },
        IRInstr::LoadFPRelative {
            from: layout.local_var_entry(1),
            to: ptr,
            size: Size::pointer()
        },
        IRInstr::LoadFPRelative {
            from: layout.local_var_entry(2),
            to: val,
            size: Size::long()
        },
        IRInstr::Store {
            to_address: ptr,
            from: val,
            size: Size::long()
        },
        IRInstr::Return {
            return_val: None,
            temp_register_1: Register(1),
            temp_register_2: Register(2),
            temp_register_3: Register(3),
            temp_register_4: Register(4),
            frame_size: layout.full_frame_size(),
        }
    ])
}
