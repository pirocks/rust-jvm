use std::ops::Deref;
use std::sync::Arc;

use by_address::ByAddress;
use libc::c_void;
use libloading::Symbol;

use classfile_view::view::HasAccessFlags;
use classfile_view::view::method_view::MethodView;
use runtime_class_stuff::RuntimeClass;

use crate::{JVMState, NewJavaValue, WasException};
use crate::better_java_stack::frames::PushableFrame;
use crate::better_java_stack::native_frame::NativeFrame;
use crate::class_loading::{assert_inited_or_initing_class, check_initing_or_inited_class};
use crate::interpreter::common::invoke::native::unsafe_temp::shouldBeInitialized;
use crate::interpreter::monitor_for_function;
use crate::new_java_values::NewJavaValueHandle;
use crate::rust_jni::{call_impl, mangling};
use crate::stack_entry::StackEntryPush;
use crate::utils::throw_npe_res;

pub fn correct_args<'gc, 'l>(args: &'l [NewJavaValue<'gc, 'l>]) -> Vec<NewJavaValue<'gc, 'l>> {
    let mut res = vec![];
    for arg in args {
        res.push(arg.clone());
        match arg {
            NewJavaValue::Long(_) => {
                res.push(NewJavaValue::Top)
            }
            NewJavaValue::Double(_) => {
                res.push(NewJavaValue::Top)
            }
            NewJavaValue::Top => {
                res.pop();
            }
            _ => {}
        }
    }
    res
}

pub struct NativeMethodWasException {
    pub prev_rip: *const c_void,
}

pub fn run_native_method<'gc, 'l, 'k>(
    jvm: &'gc JVMState<'gc>,
    int_state: &mut impl PushableFrame<'gc>,
    class: Arc<RuntimeClass<'gc>>,
    method_i: u16,
    args: Vec<NewJavaValue<'gc, 'k>>,
) -> Result<Option<NewJavaValueHandle<'gc>>, WasException<'gc>> {
    let view = &class.view();
    assert_inited_or_initing_class(jvm, view.type_());
    let method = view.method_view_i(method_i);

    assert!(method.is_native());
    let monitor = monitor_for_function(jvm, int_state, &method, method.is_synchronized());
    let owned_args_clone = args.clone();
    let corrected_args = correct_args(owned_args_clone.as_slice());
    let within_frame = |native_frame: &mut NativeFrame<'gc, '_>| {
        if let Some(m) = monitor.as_ref() {
            m.lock(jvm, native_frame).unwrap();
        }

        let result = match native_method_resolve(jvm, class.clone(), &method) {
            None => {
                match special_call_overrides(jvm, native_frame, &class.view().method_view_i(method_i), args) {
                    Ok(res) => res,
                    Err(WasException { exception_obj }) => return Err(WasException { exception_obj }),
                }
            }
            Some(res_fn) => {
                match call_impl(jvm, native_frame, class.clone(), args, method.desc().clone(), &res_fn, !method.is_static()) {
                    Ok(call_res) => call_res,
                    Err(WasException { exception_obj }) => {
                        return Err(WasException { exception_obj });
                    }
                }
            }
        };

        Ok(result)
    };
    match int_state.push_frame_native(StackEntryPush::new_native_frame(jvm, class.clone(), method_i as u16, corrected_args),
                                      within_frame) {
        Ok(res) => {
            Ok(res)
        }
        Err(WasException { exception_obj }) => {
            Err(WasException { exception_obj })
        }
    }
}

pub fn native_method_resolve<'gc>(jvm: &'gc JVMState<'gc>, class: Arc<RuntimeClass<'gc>>, method: &MethodView) -> Option<unsafe extern "C" fn()> {
    let method_i = method.method_i();
    if jvm.native_libaries.registered_natives.read().unwrap().contains_key(&ByAddress(class.clone())) && jvm.native_libaries.registered_natives.read().unwrap().get(&ByAddress(class.clone())).unwrap().read().unwrap().contains_key(&(method_i as u16)) {
        let reg_natives = jvm.native_libaries.registered_natives.read().unwrap();
        let reg_natives_for_class = reg_natives.get(&ByAddress(class.clone())).unwrap().read().unwrap();
        Some(*reg_natives_for_class.get(&(method_i as u16)).unwrap())
    } else {
        let mangled = mangling::mangle(&jvm.mangling_regex, &jvm.string_pool, &method);
        unsafe {
            let libraries_guard = jvm.native_libaries.native_libs.read().unwrap();
            let possible_symbol = libraries_guard.values().find_map(|native_lib| native_lib.library.get(&mangled.as_bytes()).ok());
            match possible_symbol {
                Some(symbol) => {
                    let symbol: Symbol<unsafe extern "C" fn()> = symbol;
                    Some(*symbol.deref())
                }
                None => {
                    None
                }
            }
        }
    }
}


fn special_call_overrides<'gc, 'l, 'k>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, method_view: &MethodView, args: Vec<NewJavaValue<'gc, 'k>>) -> Result<Option<NewJavaValueHandle<'gc>>, WasException<'gc>> {
    let mangled = mangling::mangle(&jvm.mangling_regex, &jvm.string_pool, method_view);
    //todo actually impl these at some point
    Ok(if &mangled == "Java_sun_misc_Unsafe_shouldBeInitialized" {
        //todo this isn't totally correct b/c there's a distinction between initialized and initializing.
        shouldBeInitialized(jvm, int_state, args)?.into()
    } else if &mangled == "Java_sun_misc_Unsafe_ensureClassInitialized" {
        let jclass = match args[1].cast_class() {
            None => {
                throw_npe_res(jvm, int_state)?;
                unreachable!()
            }
            Some(class) => class,
        };
        let ptype = jclass.as_runtime_class(jvm).cpdtype();
        check_initing_or_inited_class(jvm, int_state, ptype)?;
        None
    } else {
        dbg!(mangled);
        int_state.debug_print_stack_trace(jvm);
        panic!()
    })
}

pub mod unsafe_temp;