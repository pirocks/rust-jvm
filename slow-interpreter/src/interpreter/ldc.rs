use itertools::Either;

use classfile_view::view::constant_info_view::ConstantInfoView;
use rust_jvm_common::compressed_classfile::code::{CompressedLdc2W, CompressedLdcW};
use rust_jvm_common::compressed_classfile::compressed_types::CPDType;


use crate::{AllocatedHandle, JVMState, NewAsObjectOrJavaValue, NewJavaValueHandle, WasException};
use crate::better_java_stack::frames::{HasFrame, PushableFrame};
use crate::class_objects::get_or_create_class_object;
use crate::interpreter::PostInstructionAction;
use crate::interpreter::real_interpreter_state::{InterpreterJavaValue, RealInterpreterStateGuard};
use crate::stdlib::java::lang::string::JString;
use crate::string_intern::intern_safe;

fn load_class_constant<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, type_: &CPDType) -> Result<NewJavaValueHandle<'gc>, WasException<'gc>> {
    let object = get_or_create_class_object(jvm, *type_, int_state)?;
    Ok(NewJavaValueHandle::Object(AllocatedHandle::NormalObject(object)))
}

pub fn ldc2_w<'gc, 'l, 'k>(int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>, ldc2w: &CompressedLdc2W) -> PostInstructionAction<'gc> {
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
                cp => {
                    dbg!(cp);
                    int_state.inner().debug_print_stack_trace(jvm);
                    unimplemented!()
                }
            }
        }
        Either::Right(_ldc2w) => {
            todo!()
        }
    };
    PostInstructionAction::Next {}
}

pub fn from_constant_pool_entry<'gc, 'l, 'k>(c: &ConstantInfoView, jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>) -> Result<NewJavaValueHandle<'gc>,WasException<'gc>> {
    match &c {
        ConstantInfoView::Integer(i) => Ok(NewJavaValueHandle::Int(i.int)),
        ConstantInfoView::Float(f) => Ok(NewJavaValueHandle::Float(f.float)),
        ConstantInfoView::Long(l) => Ok(NewJavaValueHandle::Long(l.long)),
        ConstantInfoView::Double(d) => Ok(NewJavaValueHandle::Double(d.double)),
        ConstantInfoView::String(s) => {
            let jstring = JString::from_rust(jvm, int_state.inner(), s.string())?;
            let jstring = jstring.intern(jvm, int_state.inner())?;
            Ok(jstring.new_java_value_handle())
        }
        _ => panic!(),
    }
}
