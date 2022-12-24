use another_jit_vm::{IRMethodID, Register};
use another_jit_vm_ir::compiler::{IRInstr, IRLabel, Size};
use gc_memory_layout_common::frame_layout::NativeStackframeMemoryLayout;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_descriptors::CompressedMethodDescriptor;
use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CompressedParsedDescriptorType, CPDType};
use rust_jvm_common::compressed_classfile::method_names::MethodName;
use rust_jvm_common::global_consts::ADDRESS_SIZE;
use rust_jvm_common::MethodId;

use crate::compiler::CompilerLabeler;
use crate::compiler::intrinsics::sun_misc_unsafe::compare_and_swap::{intrinsic_compare_and_swap_int, intrinsic_compare_and_swap_long, intrinsic_compare_and_swap_object};
use crate::compiler::intrinsics::sun_misc_unsafe::get_raw::{unsafe_get_byte_raw, unsafe_get_long_raw};
use crate::compiler::intrinsics::sun_misc_unsafe::malloc_interface::{unsafe_allocate_memory, unsafe_free_memory};
use crate::compiler::intrinsics::sun_misc_unsafe::put_raw::unsafe_put_long;
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



pub fn sun_misc_unsafe<'gc>(resolver: &impl MethodResolver<'gc>, layout: &NativeStackframeMemoryLayout, labeler: &mut CompilerLabeler, method_id: MethodId, ir_method_id: IRMethodID, desc: &CMethodDescriptor, method_name: MethodName) -> Option<Vec<IRInstr>> {
    let compare_and_swap_long = CompressedMethodDescriptor {
        arg_types: vec![CClassName::object().into(), CPDType::LongType, CPDType::LongType, CPDType::LongType],
        return_type: CPDType::BooleanType,
    };
    if method_name == MethodName::method_compareAndSwapLong() && desc == &compare_and_swap_long {
        return intrinsic_compare_and_swap_long(resolver, layout, labeler, method_id, ir_method_id);
    }
    let compare_and_swap_int = CompressedMethodDescriptor {
        arg_types: vec![CPDType::object(), CPDType::LongType, CPDType::IntType, CPDType::IntType],
        return_type: CPDType::BooleanType,
    };
    if method_name == MethodName::method_compareAndSwapInt() && desc == &compare_and_swap_int {
        return intrinsic_compare_and_swap_int(resolver, layout, labeler, method_id, ir_method_id);
    }

    let compare_and_swap_obj = CompressedMethodDescriptor {
        arg_types: vec![CPDType::object(), CPDType::LongType, CPDType::object(), CPDType::object()],
        return_type: CPDType::BooleanType,
    };
    if method_name == MethodName::method_compareAndSwapObject() && desc == &compare_and_swap_obj {
        return intrinsic_compare_and_swap_object(resolver, layout, labeler, method_id, ir_method_id);
    }

    let address_size_desc = CompressedMethodDescriptor::empty_args(CPDType::IntType);
    if method_name == MethodName::method_addressSize() && desc == &address_size_desc {
        return address_size(resolver, layout, method_id, ir_method_id);
    }

    let allocate_memory_desc = CompressedMethodDescriptor { arg_types: vec![CPDType::LongType], return_type: CPDType::LongType };
    if method_name == MethodName::method_allocateMemory() && desc == &allocate_memory_desc {
        return unsafe_allocate_memory(resolver, layout, method_id, ir_method_id);
    }

    let free_memory_desc = CompressedMethodDescriptor::void_return(vec![CPDType::LongType]);
    if method_name == MethodName::method_freeMemory() && desc == &free_memory_desc {
        return unsafe_free_memory(resolver, layout, method_id, ir_method_id);
    }

    None
}