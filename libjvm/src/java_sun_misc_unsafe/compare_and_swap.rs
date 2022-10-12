use std::intrinsics::atomic_cxchg_seqcst_seqcst;
use std::mem::transmute;
use std::ops::Deref;
use std::ptr::null_mut;
use std::sync::Arc;
use std::sync::atomic::AtomicPtr;

use libc::c_void;

use classfile_view::view::ClassView;
use classfile_view::view::ptype_view::PTypeView;
use jvmti_jni_bindings::{jboolean, jint, jlong, JNIEnv, jobject};
use runtime_class_stuff::RuntimeClass;
use rust_jvm_common::compressed_classfile::field_names::FieldName;

use rust_jvm_common::runtime_type::RuntimeType;
use slow_interpreter::exceptions::WasException;
use slow_interpreter::java_values::{GcManagedObject, JavaValue, Object};
use slow_interpreter::new_java_values::{NewJavaValue, NewJavaValueHandle};
use slow_interpreter::new_java_values::allocated_objects::AllocatedHandle;
use slow_interpreter::rust_jni::jni_interface::jni::{get_interpreter_state, get_state};
use slow_interpreter::rust_jni::native_util::{from_object, from_object_new, to_object};
use slow_interpreter::utils::{throw_npe, throw_npe_res};
use verification::verifier::codecorrectness::operand_stack_has_legal_length;

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_compareAndSwapInt(env: *mut JNIEnv, the_unsafe: jobject, target_obj: jobject, offset: jlong, old: jint, new: jint) -> jboolean {
    if target_obj == null_mut() {
        todo!()
    }
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    atomic_cxchg_seqcst_seqcst((target_obj as *mut c_void).offset(offset as isize) as *mut jint, old, new).1 as jboolean
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_compareAndSwapLong(env: *mut JNIEnv, the_unsafe: jobject, target_obj: jobject, offset: jlong, old: jlong, new: jlong) -> jboolean {
    if target_obj == null_mut() {
        todo!()
    }
    atomic_cxchg_seqcst_seqcst((target_obj as *mut c_void).offset(offset as isize) as *mut jlong, old, new).1 as jboolean
}


#[no_mangle]
unsafe extern "C" fn Java_sun_misc_Unsafe_compareAndSwapObject(env: *mut JNIEnv, the_unsafe: jobject, target_obj: jobject, offset: jlong, expected: jobject, new: jobject) -> jboolean {
    //todo make these intrinsics
    if target_obj == null_mut() {
        todo!()
    }
    let target = (target_obj as *mut c_void).offset(offset as isize) as *mut jobject;
    atomic_cxchg_seqcst_seqcst(target, expected, new).1 as jboolean
}

pub fn do_swap<'l, 'gc>(curval: NewJavaValue<'gc, 'l>, expected: Option<AllocatedHandle<'gc>>, new: NewJavaValue<'gc, 'l>) -> (jboolean, NewJavaValue<'gc, 'l>) {
    let should_replace = match curval.unwrap_object() {
        None => match expected {
            None => true,
            Some(_) => false,
        },
        Some(cur) => match expected {
            None => false,
            Some(expected) => {
                cur.unwrap_alloc().raw_ptr_usize() == expected.as_allocated_obj().raw_ptr_usize()
            }
        },
    };
    let mut res = curval;
    if should_replace {
        res = new;
    }
    (should_replace as jboolean, res)
}


pub unsafe fn get_obj_and_name<'gc>(
    env: *mut JNIEnv,
    the_unsafe: jobject,
    target_obj: jobject,
    offset: jlong,
) -> Result<(Arc<RuntimeClass<'gc>>, AllocatedHandle<'gc>, FieldName), WasException<'gc>> {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let (rc, field_i) = jvm.field_table.read().unwrap().lookup(transmute(offset));
    let view = rc.view();
    let field = view.field(field_i as usize);
    let field_name = field.field_name();
    let notnull = match from_object_new(jvm, target_obj) {
        None => {
            throw_npe_res(jvm, int_state)?;
            unreachable!()
        }
        Some(notnull) => notnull,
    };
    Ok((rc, notnull, field_name))
}
