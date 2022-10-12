use itertools::Either;

use classfile_view::view::constant_info_view::ConstantInfoView;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::CPDType;
use rust_jvm_common::compressed_classfile::code::{CompressedLdc2W, CompressedLdcW};

use crate::{AllocatedHandle, JVMState, NewAsObjectOrJavaValue, NewJavaValueHandle, WasException};
use crate::better_java_stack::frames::PushableFrame;
use crate::class_objects::get_or_create_class_object;
use crate::interpreter::PostInstructionAction;
use crate::interpreter::real_interpreter_state::{InterpreterJavaValue, RealInterpreterStateGuard};
use crate::rust_jni::jni_interface::string::intern_safe;
use crate::stdlib::java::lang::string::JString;

fn load_class_constant<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, type_: &CPDType) -> Result<NewJavaValueHandle<'gc>, WasException<'gc>> {
    let object = get_or_create_class_object(jvm, *type_, int_state)?;
    Ok(NewJavaValueHandle::Object(AllocatedHandle::NormalObject(object)))
}

//
// fn load_string_constant(jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc,'l>, s: &StringView) {
//     let res_string = s.string();
//     assert!(int_state.throw().is_none());
//     let before_intern = JString::from_rust(jvm, pushable_frame_todo(), res_string).expect("todo");
//     let string = intern_safe(jvm, before_intern.object().into());
//     int_state.push_current_operand_stack(string.java_value());
// }
//
// pub fn create_string_on_stack(jvm: &'gc JVMState<'gc>, interpreter_state: &'_ mut InterpreterStateGuard<'gc,'l>, res_string: String) -> Result<(), WasException<'gc>> {
//     let java_lang_string = CClassName::string();
//     let string_class = assert_inited_or_initing_class(jvm, java_lang_string.into());
//     let str_as_vec = res_string.chars();
//     let chars: Vec<JavaValue<'gc>> = str_as_vec.map(|x| JavaValue::Char(x as u16)).collect();
//     let string_object = new_object(jvm, interpreter_state, &string_class);
//     let mut args = vec![string_object.clone()];
//     args.push(JavaValue::Object(Some(jvm.allocate_object(Object::Array(ArrayObject::new_array(jvm, interpreter_state, chars, CPDType::CharType, jvm.thread_state.new_monitor("monitor for a string".to_string()))?)))));
//     let char_array_type = CPDType::Ref(CPRefType::Array(CPDType::CharType.into()));
//     let expected_descriptor = CMethodDescriptor { arg_types: vec![char_array_type], return_type: CPDType::VoidType };
//     let (constructor_i, final_target_class) = find_target_method(jvm, interpreter_state, MethodName::constructor_init(), &expected_descriptor, string_class);
//     let next_entry = StackEntry::new_java_frame(jvm, final_target_class, constructor_i as u16, args);
//     let mut function_call_frame = interpreter_state.push_frame(next_entry);
//     match run_function(jvm, interpreter_state, &mut function_call_frame) {
//         Ok(_) => {}
//         Err(_) => todo!(),
//     }
//     let was_exception = interpreter_state.throw().is_some();
//     interpreter_state.pop_frame(jvm, function_call_frame, was_exception);
//     if !jvm.config.compiled_mode_active {
//     }
//     if interpreter_state.throw().is_some() {
//         unimplemented!()
//     }
//     if interpreter_state.function_return() {
//         interpreter_state.set_function_return(false);
//     }
//     interpreter_state.push_current_operand_stack(JavaValue::Object(string_object.unwrap_object()));
//     Ok(())
// }
//
pub fn ldc2_w<'gc, 'l, 'k>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>, ldc2w: &CompressedLdc2W) -> PostInstructionAction<'gc> {
    let mut current_frame = int_state.current_frame_mut();
    match ldc2w {
        CompressedLdc2W::Long(l) => {
            current_frame.push(InterpreterJavaValue::Long(*l));
        }
        CompressedLdc2W::Double(d) => {
            current_frame.push(InterpreterJavaValue::Double(*d));
        }
    }
    PostInstructionAction::Next {}
}

pub fn ldc_w<'gc, 'l, 'k>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>, either: &Either<&CompressedLdcW, &CompressedLdc2W>) -> PostInstructionAction<'gc> {
    match either {
        Either::Left(ldcw) => {
            match &ldcw {
                CompressedLdcW::String { str } => {
                    let jstring = JString::from_rust(jvm, int_state.inner(), str.clone()).expect("todo");
                    let string_value = intern_safe(jvm, AllocatedHandle::NormalObject(jstring.object()));
                    int_state.current_frame_mut().push(string_value.new_java_value_handle().to_interpreter_jv())
                }
                CompressedLdcW::Class { type_ } => {
                    match load_class_constant(jvm, int_state.inner(), type_) {
                        Err(WasException { exception_obj }) => {
                            return PostInstructionAction::Exception { exception: WasException { exception_obj } };
                        }
                        Ok(res) => {
                            int_state.current_frame_mut().push(res.to_interpreter_jv());
                        }
                    }
                }
                CompressedLdcW::Float { float } => {
                    let float: f32 = *float;
                    int_state.current_frame_mut().push(InterpreterJavaValue::Float(float));
                }
                CompressedLdcW::Integer { integer } => {
                    let int: i32 = *integer;
                    int_state.current_frame_mut().push(InterpreterJavaValue::Int(int));
                }
                CompressedLdcW::LiveObject(live_object_index) => {
                    let classes_guard = jvm.classes.read().unwrap();
                    let obj = classes_guard.lookup_live_object_pool(live_object_index);
                    int_state.current_frame_mut().push(obj.new_java_value_handle().to_interpreter_jv());
                }
                _ => {
                    // dbg!(cp);
                    todo!();/*int_state.inner().debug_print_stack_trace(jvm);*/
                    // dbg!(&pool_entry);
                    unimplemented!()
                }
            }
        }
        Either::Right(ldc2w) => {
            todo!()
        }
    };
    PostInstructionAction::Next {}
}

pub fn from_constant_pool_entry<'gc, 'l, 'k>(c: &ConstantInfoView, jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>) -> NewJavaValueHandle<'gc> {
    match &c {
        ConstantInfoView::Integer(i) => NewJavaValueHandle::Int(i.int),
        ConstantInfoView::Float(f) => NewJavaValueHandle::Float(f.float),
        ConstantInfoView::Long(l) => NewJavaValueHandle::Long(l.long),
        ConstantInfoView::Double(d) => NewJavaValueHandle::Double(d.double),
        ConstantInfoView::String(s) => {
            // load_string_constant(jvm, int_state, s);
            todo!();
            let string_value = int_state.current_frame_mut().pop(CClassName::string().into()).to_new_java_handle(jvm);
            intern_safe(jvm, AllocatedHandle::NormalObject(string_value.cast_string().unwrap().object())).new_java_value_handle()
        }
        _ => panic!(),
    }
}
