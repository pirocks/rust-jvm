use jvmti_jni_bindings::{jboolean, jclass, JNIEnv, jobject};
use slow_interpreter::better_java_stack::frames::PushableFrame;
use slow_interpreter::exceptions::WasException;
use slow_interpreter::java_values::ExceptionReturn;
use slow_interpreter::jvm_state::JVMState;
use slow_interpreter::new_java_values::owned_casts::OwnedCastAble;
use slow_interpreter::rust_jni::jni_utils::{get_interpreter_state, get_state, get_throw};
use slow_interpreter::rust_jni::native_util::from_object_new;
use slow_interpreter::stdlib::java::lang::class::JClass;
use slow_interpreter::utils::{unwrap_or_npe};
use runtime_class_stuff::ClassStatus;
use slow_interpreter::class_loading::check_initing_or_inited_class;
use slow_interpreter::throw_utils::throw_npe_res;

//
//todo this isn't totally correct b/c there's a distinction between initialized and initializing.

#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_shouldBeInitialized(env: *mut JNIEnv, _the_unsafe: jobject, class: jclass) -> jboolean {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    match should_be_initialized(jvm, int_state, from_object_new(jvm, class).map(|obj| obj.cast_class())) {
        Ok(res) => {
            res
        }
        Err(err) => {
            *get_throw(env) = Some(err);
            return jboolean::invalid_default();
        }
    }
}

pub fn should_be_initialized<'l, 'gc>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, class: Option<JClass<'gc>>) -> Result<jboolean, WasException<'gc>> {
    let class_to_check = unwrap_or_npe(jvm, int_state, class)?.as_runtime_class(jvm);
    let is_init = matches!(class_to_check.status(), ClassStatus::INITIALIZED);
    Ok(is_init as u8)
}



#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_ensureClassInitialized(env: *mut JNIEnv, _the_unsafe: jobject, class: jclass)  {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    if let Err(err) = ensure_initialized(jvm, int_state, from_object_new(jvm, class).map(|obj| obj.cast_class())) {
        *get_throw(env) = Some(err);
    }
}

pub fn ensure_initialized<'gc>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, class: Option<JClass<'gc>>) -> Result<(), WasException<'gc>> {
    let jclass = match class {
        None => {
            throw_npe_res(jvm, int_state)?;
            unreachable!()
        }
        Some(class) => class,
    };
    let ptype = jclass.as_runtime_class(jvm).cpdtype();
    check_initing_or_inited_class(jvm, int_state, ptype)?;
    Ok(())
}
