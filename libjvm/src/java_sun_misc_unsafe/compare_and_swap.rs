use std::mem::transmute;
use std::ops::Deref;
use std::sync::Arc;

use classfile_view::view::ClassView;
use classfile_view::view::ptype_view::PTypeView;
use jvmti_jni_bindings::{jboolean, jint, jlong, JNIEnv, jobject};
use slow_interpreter::interpreter::WasException;
use slow_interpreter::java_values::{JavaValue, Object};
use slow_interpreter::runtime_class::RuntimeClass;
use slow_interpreter::rust_jni::native_util::{from_object, get_interpreter_state, get_state, to_object};
use slow_interpreter::utils::{throw_npe, throw_npe_res};
use verification::verifier::codecorrectness::operand_stack_has_legal_length;

unsafe fn get_obj_and_name(env: *mut JNIEnv, the_unsafe: jobject, target_obj: jobject, offset: jlong) -> Result<(Arc<RuntimeClass<'gc_life>>, Arc<Object<'gc_life>>, String), WasException> {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let (rc, field_i) = jvm.field_table.read().unwrap().lookup(transmute(offset));
    let view = rc.view();
    let field = view.field(field_i as usize);
    let field_name = field.field_name();
    let notnull = match from_object(target_obj) {
        None => {
            throw_npe_res(jvm, int_state)?;
            unreachable!()
        }
        Some(notnull) => notnull
    };
    Ok((rc, notnull, field_name))
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_compareAndSwapInt(env: *mut JNIEnv, the_unsafe: jobject,
                                                                 target_obj: jobject,
                                                                 offset: jlong,
                                                                 old: jint,
                                                                 new: jint,
) -> jboolean {
    let (rc, notnull, field_name) = match get_obj_and_name(env, the_unsafe, target_obj, offset) {
        Ok((rc, notnull, field_name)) => (rc, notnull, field_name),
        Err(WasException {}) => return jboolean::MAX
    };
    let normal_obj = notnull.unwrap_normal_object();
    let curval = normal_obj.get_var(rc.clone(), field_name.as_str(), PTypeView::TopType);
    (if curval.unwrap_int() == old {
        normal_obj.set_var(rc, field_name, JavaValue::Int(new), PTypeView::TopType);
        1
    } else {
        0
    }) as jboolean
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_compareAndSwapLong(env: *mut JNIEnv, the_unsafe: jobject,
                                                                  target_obj: jobject,
                                                                  offset: jlong,
                                                                  old: jlong,
                                                                  new: jlong,
) -> jboolean {
    let (rc, notnull, field_name) = match get_obj_and_name(env, the_unsafe, target_obj, offset) {
        Ok((rc, notnull, field_name)) => (rc, notnull, field_name),
        Err(WasException {}) => return jboolean::MAX
    };
    let normal_obj = notnull.unwrap_normal_object();
    let curval = normal_obj.get_var_top_level(field_name.as_str());
    (if curval.unwrap_long() == old {
        normal_obj.set_var_top_level(field_name, JavaValue::Long(new));
        1
    } else {
        0
    }) as jboolean
}


#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_compareAndSwapObject(
    env: *mut JNIEnv,
    the_unsafe: jobject,
    target_obj: jobject,
    offset: jlong,
    old: jobject,
    new: jobject,
) -> jboolean {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let notnull = match from_object(target_obj) {
        None => {
            return throw_npe(jvm, int_state);
        }
        Some(notnull) => notnull
    };
    let new = JavaValue::Object(todo!()/*from_object(new)*/);
    match notnull.deref() {
        Object::Array(arr) => {
            let mut ref_mut = arr.unwrap_mut();
            let curval = ref_mut.get_mut((offset as usize)).unwrap();
            let old = from_object(old);
            do_swap(curval, old, new)
        }
        Object::Object(normal_obj) => {
            let (rc, notnull, field_name) = match get_obj_and_name(env, the_unsafe, target_obj, offset) {
                Ok((rc, notnull, field_name)) => (rc, notnull, field_name),
                Err(WasException {}) => return jboolean::MAX
            };
            let mut curval = normal_obj.get_var_top_level(field_name.as_str());
            let old = from_object(old);
            let res = do_swap(&mut curval, old, new);
            normal_obj.set_var_top_level(field_name, curval);
            res
        }
    }
}


pub fn do_swap(curval: &mut JavaValue<'gc_life>, old: Option<Arc<Object<'gc_life>>>, new: JavaValue<'gc_life>) -> jboolean {
    let should_replace = match curval.unwrap_object() {
        None => {
            match old {
                None => true,
                Some(_) => false
            }
        }
        Some(cur) => {
            match old {
                None => false,
                Some(old) => Arc::ptr_eq(&cur, &old)
            }
        }
    };
    if should_replace {
        *curval = new;
    }
    should_replace as jboolean
}