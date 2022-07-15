use std::sync::Arc;

use by_address::ByAddress;
use another_jit_vm_ir::WasException;

use classfile_view::view::HasAccessFlags;
use classfile_view::view::method_view::MethodView;

use crate::{InterpreterStateGuard, JVMState, NewAsObjectOrJavaValue, NewJavaValue};
use crate::class_loading::{assert_inited_or_initing_class, check_initing_or_inited_class};
use crate::instructions::invoke::native::mhn_temp::init::MHN_init;
use crate::interpreter::{monitor_for_function};
use crate::java::nio::heap_byte_buffer::HeapByteBuffer;
use crate::new_java_values::NewJavaValueHandle;
use runtime_class_stuff::RuntimeClass;
use crate::instructions::invoke::native::mhn_temp::{Java_java_lang_invoke_MethodHandleNatives_getMembers, Java_java_lang_invoke_MethodHandleNatives_objectFieldOffset, MHN_getConstant};
use crate::instructions::invoke::native::mhn_temp::resolve::MHN_resolve;
use crate::instructions::invoke::native::unsafe_temp::shouldBeInitialized;
use crate::rust_jni::{call, call_impl, mangling};
use crate::stack_entry::StackEntryPush;
use crate::utils::throw_npe_res;

pub fn correct_args<'gc, 'l>(args: &'l [NewJavaValue<'gc,'l>]) -> Vec<NewJavaValue<'gc,'l>>{
    let mut res = vec![];
    for arg in args{
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

pub fn run_native_method<'gc, 'l, 'k>(
    jvm: &'gc JVMState<'gc>,
    int_state: &'_ mut InterpreterStateGuard<'gc,'l>,
    class: Arc<RuntimeClass<'gc>>,
    method_i: u16,
    args: Vec<NewJavaValue<'gc,'k>>
) -> Result<Option<NewJavaValueHandle<'gc>>, WasException> {
    let view = &class.view();
    assert_inited_or_initing_class(jvm, view.type_());
    let method = view.method_view_i(method_i);
    assert!(method.is_native());
    let method_as_string = method.name().0.to_str(&jvm.string_pool);
    let native_call_frame = int_state.push_frame(StackEntryPush::new_native_frame(jvm, class.clone(), method_i as u16, correct_args(args.as_slice())));
    assert!(int_state.current_frame().is_native_method());
    let monitor = monitor_for_function(jvm, int_state, &method, method.is_synchronized());
    if let Some(m) = monitor.as_ref() {
        m.lock(jvm, int_state).unwrap();
    }

    let result = if jvm.native_libaries.registered_natives.read().unwrap().contains_key(&ByAddress(class.clone())) && jvm.native_libaries.registered_natives.read().unwrap().get(&ByAddress(class.clone())).unwrap().read().unwrap().contains_key(&(method_i as u16)) {
        //todo dup
        let res_fn = {
            let reg_natives = jvm.native_libaries.registered_natives.read().unwrap();
            let reg_natives_for_class = reg_natives.get(&ByAddress(class.clone())).unwrap().read().unwrap();
            *reg_natives_for_class.get(&(method_i as u16)).unwrap()
        };
        match call_impl(jvm, int_state, class.clone(), args, method.desc().clone(), &res_fn, !method.is_static()) {
            Ok(call_res) => call_res,
            Err(WasException {}) => {
                assert!(int_state.throw().is_some());
                int_state.pop_frame(jvm, native_call_frame, true);
                return Err(WasException);
            }
        }
    } else {
        assert!(int_state.current_frame().is_native_method());
        let first_call = match call(jvm, int_state, class.clone(), method.clone(), args.clone(), method.desc().clone()) {
            Ok(call_res) => call_res,
            Err(WasException {}) => {
                int_state.pop_frame(jvm, native_call_frame, true);
                assert!(int_state.throw().is_some());
                return Err(WasException)
            }
        };
        match first_call {
            Some(r) => r,
            None => match special_call_overrides(jvm, int_state, &class.view().method_view_i(method_i), args) {
                Ok(res) => res,
                Err(WasException{}) => None,
            },
        }
    };
    if let Some(m) = monitor.as_ref() {
        m.unlock(jvm, int_state).unwrap();
    }
    let was_exception = int_state.throw().is_some();
    int_state.pop_frame(jvm, native_call_frame, was_exception);
    if was_exception {
        Err(WasException)
    } else {
        Ok(result)
    }
}

fn special_call_overrides<'gc, 'l, 'k>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc,'l>, method_view: &MethodView, args: Vec<NewJavaValue<'gc,'k>>) -> Result<Option<NewJavaValueHandle<'gc>>, WasException> {
    let mangled = mangling::mangle(&jvm.string_pool, method_view);
    //todo actually impl these at some point
    Ok(if &mangled == "Java_java_lang_invoke_MethodHandleNatives_registerNatives" {
        //todo
        None
    } else if &mangled == "Java_java_lang_invoke_MethodHandleNatives_getConstant" {
        MHN_getConstant()?.into()
    } else if &mangled == "Java_java_lang_invoke_MethodHandleNatives_resolve" {
        MHN_resolve(jvm, int_state, args)?.into()
    } else if &mangled == "Java_java_lang_invoke_MethodHandleNatives_init" {
        MHN_init(jvm, int_state, args)?;
        None
    } else if &mangled == "Java_sun_misc_Unsafe_shouldBeInitialized" {
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
    } else if &mangled == "Java_java_lang_invoke_MethodHandleNatives_objectFieldOffset" {
        Java_java_lang_invoke_MethodHandleNatives_objectFieldOffset(jvm, int_state, args)?.into()
    } else if &mangled == "Java_java_lang_invoke_MethodHandleNatives_getMembers" {
        Java_java_lang_invoke_MethodHandleNatives_getMembers(jvm, int_state, args)?.into()
    } else if &mangled == "Java_sun_misc_Unsafe_putObjectVolatile" {
        unimplemented!()
    } else if &mangled == "Java_sun_misc_Perf_registerNatives" {
        //todo not really sure what to do here, for now nothing
        None
    } else if &mangled == "Java_sun_misc_Perf_createLong" {
        Some(HeapByteBuffer::new(jvm, int_state, vec![0, 0, 0, 0, 0, 0, 0, 0], 0, 8)?.new_java_value_handle())
        //todo this is incorrect and should be implemented properly.
    } else if &mangled == "Java_sun_misc_Unsafe_pageSize" {
        Some(NewJavaValueHandle::Int(4096)) //todo actually get page size
    } else {
        int_state.debug_print_stack_trace(jvm);
        dbg!(mangled);
        panic!()
    })
}

pub mod mhn_temp;
pub mod unsafe_temp;