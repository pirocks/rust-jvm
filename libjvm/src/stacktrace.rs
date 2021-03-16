use std::ptr::null_mut;
use std::sync::Arc;

use by_address::ByAddress;

use classfile_view::view::attribute_view::SourceFileView;
use classfile_view::view::ClassView;
use jvmti_jni_bindings::{jint, JNI_ERR, JNIEnv, jobject, jvmtiError_JVMTI_ERROR_CLASS_LOADER_UNSUPPORTED};
use rust_jvm_common::classfile::{LineNumberTable, LineNumberTableEntry};
use slow_interpreter::interpreter::WasException;
use slow_interpreter::java::lang::stack_trace_element::StackTraceElement;
use slow_interpreter::java::lang::string::JString;
use slow_interpreter::runtime_class::RuntimeClass;
use slow_interpreter::rust_jni::native_util::{from_object, get_interpreter_state, get_state, to_object};
use slow_interpreter::utils::{throw_array_out_of_bounds, throw_illegal_arg, throw_npe, throw_npe_res};

#[no_mangle]
unsafe extern "system" fn JVM_FillInStackTrace(env: *mut JNIEnv, throwable: jobject) {
    //todo handle opaque frames properly
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let stacktrace = int_state.cloned_stack_snapshot();

    let stack_entry_objs = stacktrace.iter().map(|stack_entry| {
        let declaring_class = match stack_entry.try_class_pointer() {
            None => return Ok(None),
            Some(declaring_class) => declaring_class
        };

        let declaring_class_view = declaring_class.view();
        let method_view = declaring_class_view.method_view_i(stack_entry.method_i() as usize);
        let file = match declaring_class_view.sourcefile_attr() {
            None => {
                "unknown_source".to_string()
            }
            Some(sourcefile) => {
                sourcefile.file()
            }
        };
        let line_number = match method_view.line_number_table() {
            None => -1,
            Some(line_number_table) => {
                //todo have a lookup function for this
                let mut cur_line = -1;
                for LineNumberTableEntry { start_pc, line_number } in &line_number_table.line_number_table {
                    if (*start_pc as usize) <= stack_entry.pc() {
                        cur_line = *line_number as jint;
                    }
                }
                cur_line
            }
        };
        let declaring_class_name = JString::from_rust(jvm, int_state, declaring_class_view.type_().class_name_representation())?;
        let method_name = JString::from_rust(jvm, int_state, method_view.name())?;
        let source_file_name = JString::from_rust(jvm, int_state, file)?;

        Ok(Some(StackTraceElement::new(jvm, int_state, declaring_class_name, method_name, source_file_name, line_number)?))
    }).collect::<Result<Vec<Option<_>>, WasException>>().expect("todo").into_iter().flatten().collect::<Vec<_>>();
    let mut stack_traces_guard = jvm.stacktraces_by_throwable.write().unwrap();
    stack_traces_guard.insert(ByAddress(match from_object(throwable) {
        Some(x) => x,
        None => {
            throw_npe(jvm, int_state);
            return;
        }
    }), stack_entry_objs);
}

#[no_mangle]
unsafe extern "system" fn JVM_GetStackTraceDepth(env: *mut JNIEnv, throwable: jobject) -> jint {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    match jvm.stacktraces_by_throwable.read().unwrap().get(&ByAddress(match from_object(throwable) {
        Some(x) => x,
        None => {
            throw_npe(jvm, int_state);
            return i32::MAX;
        }
    })) {
        Some(x) => x,
        None => return JNI_ERR,
    }.len() as i32
}

#[no_mangle]
unsafe extern "system" fn JVM_GetStackTraceElement(env: *mut JNIEnv, throwable: jobject, index: jint) -> jobject {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    match match jvm.stacktraces_by_throwable.read().unwrap().get(&ByAddress(match from_object(throwable) {
        Some(x) => x,
        None => {
            throw_npe(jvm, int_state);
            return null_mut();
        }
    })) {
        Some(x) => x,
        None => {
            throw_illegal_arg(jvm, int_state);
            return null_mut();
        }
    }.get(index as usize) {
        None => {
            throw_array_out_of_bounds(jvm, int_state, index);
            return null_mut();
        }
        Some(element) => to_object(element.clone().object().into())
    }
}

