use std::mem::transmute;
use std::ops::Deref;
use std::sync::Arc;

use classfile_view::view::ClassView;
use classfile_view::view::ptype_view::PTypeView;
use jvmti_jni_bindings::{jboolean, jint, jlong, JNIEnv, jobject};
use rust_jvm_common::compressed_classfile::names::FieldName;
use rust_jvm_common::runtime_type::RuntimeType;
use slow_interpreter::interpreter::WasException;
use slow_interpreter::java_values::{GcManagedObject, JavaValue, Object};
use slow_interpreter::new_java_values::{AllocatedObjectHandle, NewJavaValue, NewJavaValueHandle};
use slow_interpreter::runtime_class::RuntimeClass;
use slow_interpreter::rust_jni::native_util::{from_object, from_object_new, get_interpreter_state, get_state, to_object};
use slow_interpreter::utils::{throw_npe, throw_npe_res};
use verification::verifier::codecorrectness::operand_stack_has_legal_length;


#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_compareAndSwapInt(env: *mut JNIEnv, the_unsafe: jobject, target_obj: jobject, offset: jlong, old: jint, new: jint) -> jboolean {
    let jvm = get_state(env);
    let (rc, notnull, field_name) = match get_obj_and_name(env, the_unsafe, target_obj, offset) {
        Ok((rc, notnull, field_name)) => (rc, notnull, field_name),
        Err(WasException {}) => return jboolean::MAX,
    };
    let curval = notnull.as_allocated_obj().get_var(jvm, &rc, field_name);
    (if curval.as_njv().unwrap_int() == old {
        notnull.as_allocated_obj().set_var(&rc, field_name, NewJavaValue::Int(new));
        1
    } else {
        0
    }) as jboolean
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_compareAndSwapLong(env: *mut JNIEnv, the_unsafe: jobject, target_obj: jobject, offset: jlong, old: jlong, new: jlong) -> jboolean {
    let jvm = get_state(env);
    let (rc, notnull, field_name) = match get_obj_and_name(env, the_unsafe, target_obj, offset) {
        Ok((rc, notnull, field_name)) => (rc, notnull, field_name),
        Err(WasException {}) => return jboolean::MAX,
    };
    let jv = notnull.to_jv();
    let normal_obj = jv.unwrap_normal_object();
    let curval = normal_obj.get_var_top_level(jvm, field_name);
    (if curval.unwrap_long() == old {
        normal_obj.set_var(rc, field_name, JavaValue::Long(new));
        1
    } else {
        0
    }) as jboolean
}



#[no_mangle]
unsafe extern "C" fn Java_sun_misc_Unsafe_compareAndSwapObject(env: *mut JNIEnv, the_unsafe: jobject, target_obj: jobject, offset: jlong, expected: jobject, new: jobject) -> jboolean {
    dbg!(the_unsafe);
    dbg!(target_obj);
    dbg!(offset);
    dbg!(expected);
    dbg!(new);
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    int_state.debug_print_stack_trace(jvm);
    let notnull = match from_object_new(jvm, target_obj) {
        None => {
            return throw_npe(jvm, int_state);
        }
        Some(notnull) => notnull,
    };
    let new = NewJavaValueHandle::from_optional_object(from_object_new(jvm, new));
    if notnull.is_array(jvm) {
        todo!()
        /*let curval = arr.get_i(jvm, (offset as i32));
        let old = from_object(jvm, old);
        let (should_replace, new) = do_swap(curval, old, new);
        todo!();
        // arr.set_i(jvm, offset as i32, new);
        should_replace*/
    } else {
        let (rc, notnull, field_name) = match get_obj_and_name(env, the_unsafe, target_obj, offset) {
            Ok((rc, notnull, field_name)) => (rc, notnull, field_name),
            Err(WasException {}) => return jboolean::MAX,
        };
        dbg!(field_name.0.to_str(&jvm.string_pool));
        let curval = notnull.as_allocated_obj().get_var(jvm, &rc, field_name);
        let expected = from_object_new(jvm, expected);
        let (should_replace, new) = do_swap(curval.as_njv(), expected, new.as_njv());
        notnull.as_allocated_obj().set_var(&rc, field_name, new);
        dbg!(should_replace)
    }
}

pub fn do_swap<'l, 'gc_life>(curval: NewJavaValue<'gc_life, 'l>, expected: Option<AllocatedObjectHandle<'gc_life>>, new: NewJavaValue<'gc_life, 'l>) -> (jboolean, NewJavaValue<'gc_life, 'l>) {
    let should_replace = match curval.unwrap_object() {
        None => match expected {
            None => true,
            Some(_) => false,
        },
        Some(cur) => match expected {
            None => false,
            Some(expected) => {
                dbg!(cur.unwrap_alloc().raw_ptr_usize()) == dbg!(expected.as_allocated_obj().raw_ptr_usize())
            }
        },
    };
    let mut res = curval;
    if should_replace {
        res = new;
    }
    (should_replace as jboolean, res)
}


pub unsafe fn get_obj_and_name<'gc_life>(
    env: *mut JNIEnv,
    the_unsafe: jobject,
    target_obj: jobject,
    offset: jlong,
) -> Result<(Arc<RuntimeClass<'gc_life>>, AllocatedObjectHandle<'gc_life>, FieldName), WasException> {
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
