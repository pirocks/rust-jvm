use itertools::Either;

use classfile_view::view::constant_info_view::{ConstantInfoView, StringView};
use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType, CPRefType};
use rust_jvm_common::compressed_classfile::code::{CompressedLdc2W, CompressedLdcW};
use rust_jvm_common::compressed_classfile::names::{CClassName, MethodName};

use crate::{InterpreterStateGuard, JVMState, NewJavaValue, StackEntry};
use crate::class_loading::assert_inited_or_initing_class;
use crate::class_objects::get_or_create_class_object;
use crate::instructions::invoke::find_target_method;
use crate::interpreter::{run_function, WasException};
use crate::interpreter_util::new_object;
use crate::java::lang::string::JString;
use crate::java::NewAsObjectOrJavaValue;
use crate::java_values::{ArrayObject, JavaValue, Object};
use crate::new_java_values::NewJavaValueHandle;
use crate::rust_jni::interface::string::intern_safe;
use crate::stack_entry::{StackEntryMut, StackEntryPush};

fn load_class_constant(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life,'l>, type_: &CPDType) -> Result<NewJavaValue<'gc_life,'gc_life>, WasException> {
    load_class_constant_by_type(jvm, int_state, type_)
}

pub fn load_class_constant_by_type(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life,'l>, res_class_type: &CPDType) -> Result<NewJavaValue<'gc_life,'gc_life>, WasException> {
    let object = get_or_create_class_object(jvm, res_class_type.clone(), int_state)?;
    Ok(NewJavaValue::AllocObject(object))
}

fn load_string_constant(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life,'l>, s: &StringView) -> NewJavaValueHandle<'gc_life>{
    let res_string = s.string();
    assert!(int_state.throw().is_none());
    let before_intern = JString::from_rust(jvm, int_state, res_string).expect("todo");
    let string = intern_safe(jvm, before_intern.object().into());
    string.new_java_value_handle()
}

pub fn create_string_on_stack(jvm: &'gc_life JVMState<'gc_life>, interpreter_state: &'_ mut InterpreterStateGuard<'gc_life,'l>, res_string: String) -> Result<(), WasException> {
    let java_lang_string = CClassName::string();
    let string_class = assert_inited_or_initing_class(jvm, java_lang_string.into());
    let str_as_vec = res_string.chars();
    let chars: Vec<JavaValue<'gc_life>> = str_as_vec.map(|x| JavaValue::Char(x as u16)).collect();
    let string_object = new_object(jvm, interpreter_state, &string_class).to_jv();
    let mut args = vec![string_object.clone()];
    args.push(JavaValue::Object(todo!()/*Some(jvm.allocate_object(todo!()/*Object::Array(ArrayObject::new_array(jvm, interpreter_state, chars, CPDType::CharType, jvm.thread_state.new_monitor("monitor for a string".to_string()))?)*/))*/));
    let char_array_type = CPDType::Ref(CPRefType::Array(CPDType::CharType.into()));
    let expected_descriptor = CMethodDescriptor { arg_types: vec![char_array_type], return_type: CPDType::VoidType };
    let (constructor_i, final_target_class) = find_target_method(jvm, interpreter_state, MethodName::constructor_init(), &expected_descriptor, string_class);
    let next_entry = StackEntryPush::new_java_frame(jvm, final_target_class, constructor_i as u16, todo!()/*args*/);
    let mut function_call_frame = interpreter_state.push_frame(next_entry);
    match run_function(jvm, interpreter_state, &mut function_call_frame) {
        Ok(_) => {}
        Err(_) => todo!(),
    }
    let was_exception = interpreter_state.throw().is_some();
    interpreter_state.pop_frame(jvm, function_call_frame, was_exception);
    if !jvm.config.compiled_mode_active {
    }
    if interpreter_state.throw().is_some() {
        unimplemented!()
    }
    if interpreter_state.function_return() {
        interpreter_state.set_function_return(false);
    }
    interpreter_state.push_current_operand_stack(JavaValue::Object(string_object.unwrap_object()));
    Ok(())
}

pub fn ldc2_w(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, ldc2w: &CompressedLdc2W) {
    match ldc2w {
        CompressedLdc2W::Long(l) => {
            current_frame.push(JavaValue::Long(*l));
        }
        CompressedLdc2W::Double(d) => {
            current_frame.push(JavaValue::Double(*d));
        }
        _ => {}
    }
}

pub fn ldc_w(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life,'l>, either: &Either<&CompressedLdcW, &CompressedLdc2W>) {
    match either {
        Either::Left(ldcw) => {
            match &ldcw {
                CompressedLdcW::String { str } => {
                    let string_value = intern_safe(jvm, todo!()/*JString::from_rust(jvm, int_state, str.clone()).expect("todo").object().to_gc_managed().into()*/).java_value();
                    int_state.push_current_operand_stack(string_value)
                }
                CompressedLdcW::Class { type_ } => {
                    if let Err(WasException {}) = load_class_constant(jvm, int_state, type_) {
                        return;
                    }
                }
                CompressedLdcW::Float { float } => {
                    let float: f32 = *float;
                    int_state.push_current_operand_stack(JavaValue::Float(float));
                }
                CompressedLdcW::Integer { integer } => {
                    let int: i32 = *integer;
                    int_state.push_current_operand_stack(JavaValue::Int(int));
                }
                CompressedLdcW::LiveObject(live_object_index) => {
                    let classes_guard = jvm.classes.read().unwrap();
                    let obj = classes_guard.lookup_live_object_pool(live_object_index);
                    int_state.push_current_operand_stack(JavaValue::Object(todo!()/*Some(obj.clone())*/));
                }
                _ => {
                    // dbg!(cp);
                    int_state.debug_print_stack_trace(jvm);
                    // dbg!(&pool_entry);
                    unimplemented!()
                }
            }
        }
        Either::Right(ldc2w) => {
            todo!()
        }
    }
}

pub fn from_constant_pool_entry(c: &ConstantInfoView, jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life,'l>) -> NewJavaValueHandle<'gc_life> {
    match &c {
        ConstantInfoView::Integer(i) => NewJavaValueHandle::Int(i.int),
        ConstantInfoView::Float(f) => NewJavaValueHandle::Float(f.float),
        ConstantInfoView::Long(l) => NewJavaValueHandle::Long(l.long),
        ConstantInfoView::Double(d) => NewJavaValueHandle::Double(d.double),
        ConstantInfoView::String(s) => {
            let string_value = load_string_constant(jvm, int_state, s);;
            intern_safe(jvm, string_value.unwrap_object_nonnull()).new_java_value_handle()
        }
        _ => panic!(),
    }
}