use jvmti_jni_bindings::jint;
use crate::better_java_stack::frames::PushableFrame;
use crate::exceptions::WasException;
use crate::java_values::ExceptionReturn;
use crate::jvm_state::JVMState;
use crate::new_java_values::owned_casts::OwnedCastAble;
use crate::stdlib::java::lang::array_out_of_bounds_exception::ArrayOutOfBoundsException;
use crate::stdlib::java::lang::illegal_argument_exception::IllegalArgumentException;
use crate::stdlib::java::lang::null_pointer_exception::NullPointerException;
use crate::stdlib::java::NewAsObjectOrJavaValue;

pub fn throw_npe_res<'gc, 'l, T: ExceptionReturn>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<T, WasException<'gc>> {
    let mut throw = None;
    let _ = throw_npe::<T>(jvm, int_state, &mut throw);
    Err(WasException { exception_obj: throw.unwrap().exception_obj })
}

pub fn throw_npe<'gc, 'l, T: ExceptionReturn>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, throw: &mut Option<WasException<'gc>>) -> T {
    let npe_object = match NullPointerException::new(jvm, int_state) {
        Ok(npe) => npe,
        Err(WasException { .. }) => {
            panic!("Exception occurred creating exception")
        }
    }
        .object()
        .cast_throwable();
    *throw = Some(WasException { exception_obj: npe_object });
    T::invalid_default()
}

pub fn throw_array_out_of_bounds_res<'gc, 'l, T: ExceptionReturn>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, index: jint) -> Result<T, WasException<'gc>> {
    let mut throw = None;
    let _ = throw_array_out_of_bounds::<T>(jvm, int_state, &mut throw, index);
    Err(throw.unwrap())
}

pub fn throw_array_out_of_bounds<'gc, 'l, T: ExceptionReturn>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, throw: &mut Option<WasException<'gc>>, index: jint) -> T {
    let bounds_object = match ArrayOutOfBoundsException::new(jvm, int_state, index) {
        Ok(npe) => npe,
        Err(WasException { .. }) => {
            todo!();
            eprintln!("Warning error encountered creating Array out of bounds");
            return T::invalid_default();
        }
    }
        .object()
        .new_java_handle().unwrap_object_nonnull();
    *throw = Some(WasException { exception_obj: bounds_object.cast_throwable() });
    T::invalid_default()
}

pub fn throw_illegal_arg_res<'gc, 'l, T: ExceptionReturn>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<T, WasException<'gc>> {
    let mut res = None;
    let _ = throw_illegal_arg::<T>(jvm, int_state, &mut res);
    Err(WasException { exception_obj: res.unwrap().exception_obj })
}

pub fn throw_illegal_arg<'gc, 'l, T: ExceptionReturn>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, throw: &mut Option<WasException<'gc>>) -> T {
    let illegal_arg_object = match IllegalArgumentException::new(jvm, int_state) {
        Ok(illegal_arg) => illegal_arg,
        Err(WasException { .. }) => {
            eprintln!("Warning error encountered creating illegal arg exception");
            return T::invalid_default();
        }
    }.object();
    *throw = Some(WasException { exception_obj: illegal_arg_object.cast_throwable() });
    T::invalid_default()
}


