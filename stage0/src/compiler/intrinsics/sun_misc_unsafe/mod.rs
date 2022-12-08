use another_jit_vm::{IRMethodID, Register};
use another_jit_vm_ir::compiler::{IRInstr, IRLabel, Size};
use gc_memory_layout_common::frame_layout::NativeStackframeMemoryLayout;
use rust_jvm_common::global_consts::ADDRESS_SIZE;
use rust_jvm_common::MethodId;

use crate::compiler::CompilerLabeler;
use crate::compiler_common::MethodResolver;

pub mod compare_and_swap;
pub mod get_raw;
pub mod put_raw;
pub mod malloc_interface;

// #[no_mangle]
// unsafe extern "system" fn Java_sun_misc_Unsafe_getIntVolatile(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong) -> jint {
//     let jvm = get_state(env);
//     let int_state = get_interpreter_state(env);
//     match from_object_new(jvm, obj) {
//         Some(notnull) => {
//             return volatile_load((obj as *const c_void).offset(offset as isize) as *const jint);
//         }
//         None => {
//             //static
//             let (rc, field_i) = jvm.field_table.read().unwrap().lookup(transmute(offset));
//             let field_name = rc.view().field(field_i as usize).field_name();
//             let static_vars = static_vars(rc.deref(), jvm);
//             static_vars.get(field_name).unwrap_int()
//         }
//     }
// }
pub fn get_int_volatile<'gc>(
    resolver: &impl MethodResolver<'gc>,
    layout: &NativeStackframeMemoryLayout,
    labeler: &mut CompilerLabeler,
    method_id: MethodId,
    ir_method_id: IRMethodID
) -> Option<Vec<IRInstr>> {
    let obj = Register(1);
    let zero = Register(2);
    let res = Register(3);
    let offset = Register(4);
    let static_var_lookup = labeler.local_label();
    return Some(vec![
        IRInstr::IRStart {
            temp_register: Register(2),
            ir_method_id,
            method_id,
            frame_size: layout.full_frame_size(),
            num_locals: resolver.num_locals(method_id) as usize,
        },
        IRInstr::Const64bit { to: zero, const_: 0 },
        IRInstr::LoadFPRelative {
            from: layout.local_var_entry(1),
            to: obj,
            size: Size::pointer(),
        },
        IRInstr::BranchEqual {
            a: obj,
            b: zero,
            label: static_var_lookup,
            size: Size::pointer(),
        },
        IRInstr::LoadFPRelative {
            from: layout.local_var_entry(2),
            to: offset,
            size: Size::long(),
        },
        IRInstr::Add {
            res: obj,
            a: offset,
            size: Size::pointer(),
        },
        IRInstr::Load {
            to: res,
            from_address: obj,
            size: Size::int(),
        },
        IRInstr::Return {
            return_val: Some(res),
            temp_register_1: Register(1),
            temp_register_2: Register(2),
            temp_register_3: Register(3),
            temp_register_4: Register(4),
            frame_size: layout.full_frame_size(),
        },
        IRInstr::Label(IRLabel { name: static_var_lookup }),
        IRInstr::DebuggerBreakpoint,
    ]);
}


pub fn address_size<'gc>(resolver: &impl MethodResolver<'gc>, layout: &NativeStackframeMemoryLayout, method_id: MethodId, ir_method_id: IRMethodID) -> Option<Vec<IRInstr>> {
    let res = Register(0);
    return Some(vec![
        IRInstr::IRStart {
            temp_register: Register(2),
            ir_method_id,
            method_id,
            frame_size: layout.full_frame_size(),
            num_locals: resolver.num_locals(method_id) as usize,
        },
        IRInstr::Const32bit { to: res, const_: ADDRESS_SIZE as u32 },
        IRInstr::Return {
            return_val: Some(res),
            temp_register_1: Register(1),
            temp_register_2: Register(2),
            temp_register_3: Register(3),
            temp_register_4: Register(4),
            frame_size: layout.full_frame_size(),
        },
    ]);
}
