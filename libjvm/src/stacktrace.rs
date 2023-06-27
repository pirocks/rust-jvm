use std::sync::Arc;

use itertools::Itertools;
use wtf8::Wtf8Buf;
use another_jit_vm_ir::ir_stack::IsOpaque;

use classfile_view::view::ptype_view::PTypeView;
use jvmti_jni_bindings::{jint, JNI_ERR, JNIEnv, jobject};
use runtime_class_stuff::RuntimeClass;
use rust_jvm_common::classfile::{LineNumber};
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::method_names::MethodName;
use slow_interpreter::better_java_stack::frames::HasFrame;
use slow_interpreter::class_loading::assert_loaded_class;
use slow_interpreter::exceptions::WasException;
use slow_interpreter::interpreter::common::special::inherits_from;
use slow_interpreter::new_java_values::allocated_objects::AllocatedObjectHandleByAddress;

use slow_interpreter::rust_jni::jni_utils::get_throw;
use slow_interpreter::rust_jni::native_util::{from_object_new, to_object_new};
use slow_interpreter::stdlib::java::lang::stack_trace_element::StackTraceElement;
use slow_interpreter::stdlib::java::lang::string::JString;
use slow_interpreter::stdlib::java::NewAsObjectOrJavaValue;
use slow_interpreter::utils::{lookup_line_number};
use slow_interpreter::rust_jni::jni_utils::{get_interpreter_state, get_state};
use slow_interpreter::throw_utils::{throw_array_out_of_bounds, throw_illegal_arg, throw_npe};

struct OwnedStackEntry<'gc> {
    _declaring_class: Arc<RuntimeClass<'gc>>,
    line_number: LineNumber,
    class_name_wtf8: Wtf8Buf,
    method_name_wtf8: Wtf8Buf,
    source_file_name_wtf8: Wtf8Buf,
}


#[no_mangle]
unsafe extern "system" fn JVM_FillInStackTrace<'gc>(env: *mut JNIEnv, throwable: jobject) {
    //todo handle opaque frames properly
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let stacktrace = int_state.frame_iter().collect_vec();
    let mut skipping = true;
    let stack_entry_objs = stacktrace
        .iter()
        .map(|stack_entry| {
            let declaring_class: Arc<RuntimeClass<'gc>> = match stack_entry.try_class_pointer(jvm) {
                Err(IsOpaque{}) => return Ok(None),
                Ok(declaring_class) => declaring_class,
            };

            let declaring_class_view = declaring_class.view();
            let method_view = declaring_class_view.method_view_i(stack_entry.method_i());
            if skipping{
                if method_view.name() == MethodName::method_fillInStackTrace() {
                    return Ok(None);
                }
                if inherits_from(jvm, &declaring_class, &assert_loaded_class(jvm, CClassName::throwable().into())){
                    return Ok(None);
                }
                skipping = false;
            }
            let file = match declaring_class_view.sourcefile_attr() {
                None => Wtf8Buf::from_string("unknown_source".to_string()),
                Some(sourcefile) => sourcefile.file(),
            };
            let line_number = match method_view.line_number_table() {
                None => LineNumber(u16::MAX),
                Some(line_number_table) => {
                    lookup_line_number(line_number_table, stack_entry).unwrap_or(LineNumber(u16::MAX))
                }
            };
            let class_name_wtf8 = Wtf8Buf::from_string(PTypeView::from_compressed(declaring_class_view.type_(), jvm.string_pool).class_name_representation());
            let method_name_wtf8 = Wtf8Buf::from_string(method_view.name().0.to_str(jvm.string_pool));
            let source_file_name_wtf8 = file;
            Ok(Some(OwnedStackEntry { _declaring_class: declaring_class, line_number, class_name_wtf8, method_name_wtf8, source_file_name_wtf8 }))
        })
        .collect::<Result<Vec<Option<_>>, WasException<'gc>>>()
        .expect("todo")
        .into_iter()
        .flatten()
        .map(|OwnedStackEntry { _declaring_class, line_number, class_name_wtf8, method_name_wtf8, source_file_name_wtf8 }| {
            let declaring_class_name = JString::from_rust(jvm, int_state, class_name_wtf8)?;
            let method_name = JString::from_rust(jvm, int_state, method_name_wtf8)?;
            let source_file_name = JString::from_rust(jvm, int_state, source_file_name_wtf8)?;

            StackTraceElement::new(jvm, int_state, declaring_class_name, method_name, source_file_name, line_number)
        })
        .collect::<Result<Vec<_>, WasException<'gc>>>().expect("todo");

    let mut stack_traces_guard = jvm.stacktraces_by_throwable.write().unwrap();
    stack_traces_guard.insert(
        AllocatedObjectHandleByAddress(match from_object_new(jvm, throwable) {
            Some(x) => x,
            None => {
                return throw_npe(jvm, int_state,get_throw(env));
            }
        }),
        stack_entry_objs,
    );
}

#[no_mangle]
unsafe extern "system" fn JVM_GetStackTraceDepth(env: *mut JNIEnv, throwable: jobject) -> jint {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    match jvm.stacktraces_by_throwable.read().unwrap().get(&AllocatedObjectHandleByAddress(match from_object_new(jvm, throwable) {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state,get_throw(env));
        }
    })) {
        Some(x) => x,
        None => return JNI_ERR,
    }
        .len() as i32
}

#[no_mangle]
unsafe extern "system" fn JVM_GetStackTraceElement(env: *mut JNIEnv, throwable: jobject, index: jint) -> jobject {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let throw = get_throw(env);
    let guard = jvm.stacktraces_by_throwable.read().unwrap();
    let throwable_not_null = match from_object_new(jvm, throwable) {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state,get_throw(env));
        }
    };
    let stack_traces: &Vec<StackTraceElement> = match guard.get(&AllocatedObjectHandleByAddress(throwable_not_null)) {
        Some(x) => x,
        None => {
            return throw_illegal_arg(jvm, int_state, get_throw(env));
        }
    };
    match stack_traces.get(index as usize)
    {
        None => {
            throw_array_out_of_bounds(jvm, int_state, throw, index)
        }
        Some(element) => { to_object_new(Some(element.full_object_ref())) }
    }
}
