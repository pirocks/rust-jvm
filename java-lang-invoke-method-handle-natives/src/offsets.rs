use jvmti_jni_bindings::{jclass, jlong, JNIEnv, jobject};
use slow_interpreter::better_java_stack::frames::PushableFrame;
use slow_interpreter::exceptions::WasException;
use slow_interpreter::java_values::ExceptionReturn;
use slow_interpreter::jvm_state::JVMState;
use slow_interpreter::new_java_values::{NewJavaValueHandle};
use slow_interpreter::new_java_values::java_value_common::JavaValueCommon;
use slow_interpreter::new_java_values::owned_casts::OwnedCastAble;
use slow_interpreter::rust_jni::jni_utils::{get_interpreter_state, get_state, get_throw};
use slow_interpreter::rust_jni::native_util::from_object_new;
use slow_interpreter::stdlib::java::lang::member_name::MemberName;
use slow_interpreter::stdlib::java::lang::reflect::field::Field;
use slow_interpreter::stdlib::sun::misc::unsafe_::Unsafe;
use slow_interpreter::throw_utils::throw_npe;
use slow_interpreter::utils::unwrap_or_npe;

#[no_mangle]
pub unsafe extern "system" fn Java_java_lang_invoke_MethodHandleNatives_objectFieldOffset<'gc, 'l>(env: *mut JNIEnv, _: jclass, member_name: jobject) -> jlong {

    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);

    let member_name = match from_object_new(jvm, member_name) {
        Some(member_name) => member_name.cast_member_name(),
        None => {
            return throw_npe(jvm, int_state, get_throw(env))
        },
    };

    match mhn_object_field_offset(jvm, int_state, member_name){
        Ok(res) => {
            return res;
        }
        Err(err) => {
            *get_throw(env) = Some(err);
            jlong::invalid_default()
        }
    }
}

fn mhn_object_field_offset<'gc>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, member_name: MemberName<'gc>) -> Result<jlong, WasException<'gc>> {
    let name = member_name.get_name_func(jvm, int_state)?.expect("null name?");
    let clazz = unwrap_or_npe(jvm, int_state, member_name.clazz(jvm))?;
    let field_type_option = member_name.get_field_type(jvm, int_state)?;
    let field_type = unwrap_or_npe(jvm, int_state, field_type_option)?;
    // let empty_string = JString::from_rust(jvm, int_state, Wtf8Buf::from_string("".to_string()))?;
    //todo impl annotations.
    let field = Field::init(jvm, int_state, clazz, name, field_type, 0, 0, None, NewJavaValueHandle::Null)?;
    let res = Unsafe::the_unsafe(jvm, int_state).object_field_offset(jvm, int_state, field)?;
    Ok(res.unwrap_long_strict())
}
