use std::sync::Arc;

use classfile_view::view::attribute_view::SourceFileView;
use jvmti_jni_bindings::{jint, JNIEnv, jobject};
use rust_jvm_common::classfile::{LineNumberTable, LineNumberTableEntry};
use slow_interpreter::java::lang::stack_trace_element::StackTraceElement;
use slow_interpreter::java::lang::string::JString;
use slow_interpreter::runtime_class::RuntimeClass;
use slow_interpreter::rust_jni::native_util::{from_object, get_interpreter_state, get_state};

#[no_mangle]
unsafe extern "system" fn JVM_FillInStackTrace(env: *mut JNIEnv, throwable: jobject) {
    //todo handle opaque frames properly
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let stacktrace = int_state.cloned_stack_snapshot();

    let stack_entry_objs = stacktrace.iter().flat_map(|stack_entry| {
        let declaring_class = stack_entry.try_class_pointer()?;

        let method_view = declaring_class.view().method_view_i(stack_entry.method_i() as usize);
        let file = match declaring_class.view().sourcefile_attr() {
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
                //todo have a lookup funciton for this
                let mut cur_line = -1;
                for LineNumberTableEntry { start_pc, line_number } in line_number_table.line_number_table {
                    if (start_pc as usize) <= stack_entry.pc() {
                        cur_line = line_number as jint;
                    }
                }
                cur_line
            }
        };
        let declaring_class_name = JString::from_rust(jvm, int_state, declaring_class.view().name().get_referred_name().to_string());
        let method_name = JString::from_rust(jvm, int_state, method_view.name());
        let source_file_name = JString::from_rust(jvm, int_state, file);

        Some(StackTraceElement::new(jvm, int_state, declaring_class_name, method_name, source_file_name, line_number))
    }).collect::<Vec<_>>();
    let mut stack_traces_guard = jvm.stacktraces_by_throwable.write().unwrap();
    stack_traces_guard.insert(from_object(throwable).unwrap(), stack_entry_objs);
}

#[no_mangle]
unsafe extern "system" fn JVM_GetStackTraceDepth(env: *mut JNIEnv, throwable: jobject) -> jint {
    let int_state = get_interpreter_state(env);
    0//todo impl
}

#[no_mangle]
unsafe extern "system" fn JVM_GetStackTraceElement(env: *mut JNIEnv, throwable: jobject, index: jint) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_CountStackFrames(env: *mut JNIEnv, thread: jobject) -> jint {
    unimplemented!()
}
