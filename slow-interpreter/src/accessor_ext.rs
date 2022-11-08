use std::ffi::c_void;
use std::ptr::{NonNull, null_mut};

use jvmti_jni_bindings::{jboolean, jbyte, jchar, jobject, jshort};
use runtime_class_stuff::accessor::Accessor;
use runtime_class_stuff::array_layout::ArrayAccessor;
use runtime_class_stuff::object_layout::FieldAccessor;
use runtime_class_stuff::static_fields::StaticField;
use rust_jvm_common::compressed_classfile::compressed_types::CPDType;

use crate::{JavaValueCommon, JVMState, NewJavaValue, NewJavaValueHandle};
use crate::interpreter::real_interpreter_state::InterpreterJavaValue;



pub trait AccessorExt: Accessor {
    fn write_njv<'gc, 'l>(&self, njv: NewJavaValue<'gc, 'l>, expected_type: CPDType) {
        match expected_type {
            CPDType::BooleanType => {
                self.write_boolean(njv.unwrap_int() as jboolean)//todo add an assert that there can be no other vals set?
            }
            CPDType::ByteType => {
                self.write_byte(njv.unwrap_int() as jbyte) //todo assert?
            }
            CPDType::ShortType => {
                self.write_short(njv.unwrap_int() as jshort)
            }
            CPDType::CharType => {
                self.write_char(njv.unwrap_int() as jchar);//todo assert?
            }
            CPDType::IntType => {
                self.write_int(njv.unwrap_int());
            }
            CPDType::LongType => {
                self.write_long(njv.unwrap_long_strict());
            }
            CPDType::FloatType => {
                self.write_float(njv.unwrap_float_strict());
            }
            CPDType::DoubleType => {
                self.write_double(njv.unwrap_double_strict());
            }
            CPDType::VoidType => {
                todo!()
            }
            CPDType::Class(_) | CPDType::Array { .. } => {
                //todo what about unallocated?
                match njv.unwrap_object_alloc() {
                    None => {
                        self.write_object(null_mut())
                    }
                    Some(allocated) => {
                        self.write_object(allocated.ptr().as_ptr() as jobject)
                    }
                }
            }
        }
    }

    fn read_njv<'gc>(&self, jvm: &'gc JVMState<'gc>, expected_type: CPDType) -> NewJavaValueHandle<'gc> {
        match expected_type {
            CPDType::BooleanType => {
                NewJavaValueHandle::Boolean(self.read_boolean())
            }
            CPDType::ByteType => {
                NewJavaValueHandle::Byte(self.read_byte())
            }
            CPDType::ShortType => {
                NewJavaValueHandle::Short(self.read_short())
            }
            CPDType::CharType => {
                NewJavaValueHandle::Char(self.read_char())
            }
            CPDType::IntType => {
                NewJavaValueHandle::Int(self.read_int())
            }
            CPDType::LongType => {
                NewJavaValueHandle::Long(self.read_long())
            }
            CPDType::FloatType => {
                NewJavaValueHandle::Float(self.read_float())
            }
            CPDType::DoubleType => {
                NewJavaValueHandle::Double(self.read_double())
            }
            CPDType::VoidType => {
                todo!()
            }
            CPDType::Class(_) |
            CPDType::Array { .. } => {
                match NonNull::new(self.read_object()) {
                    Some(ptr) => {
                        NewJavaValueHandle::Object(jvm.gc.register_root_reentrant(jvm, ptr.cast()))
                    },
                    None => NewJavaValueHandle::Null,
                }
            }
        }
    }

    fn read_interpreter_jv(&self, expected_type: CPDType) -> InterpreterJavaValue {
        match expected_type {
            CPDType::BooleanType => {
                InterpreterJavaValue::Int(self.read_boolean() as i32)
            }
            CPDType::ByteType => {
                InterpreterJavaValue::Int(self.read_byte() as i32)
            }
            CPDType::ShortType => {
                InterpreterJavaValue::Int(self.read_short() as i32)
            }
            CPDType::CharType => {
                InterpreterJavaValue::Int(self.read_char() as i32)
            }
            CPDType::IntType => {
                InterpreterJavaValue::Int(self.read_int())
            }
            CPDType::LongType => {
                InterpreterJavaValue::Long(self.read_long())
            }
            CPDType::FloatType => {
                InterpreterJavaValue::Float(self.read_float())
            }
            CPDType::DoubleType => {
                InterpreterJavaValue::Double(self.read_double())
            }
            CPDType::VoidType => {
                todo!()
            }
            CPDType::Class(_) |
            CPDType::Array { .. } => {
                InterpreterJavaValue::Object(NonNull::new(self.read_object() as *mut c_void))
            }
        }
    }

    fn write_interpreter_jv(&self, to_write: InterpreterJavaValue, expected_type: CPDType) {
        match expected_type {
            CPDType::BooleanType => {
                self.write_boolean(to_write.unwrap_int() as jboolean)
            }
            CPDType::ByteType => {
                self.write_byte(to_write.unwrap_int() as jbyte)
            }
            CPDType::ShortType => {
                self.write_short(to_write.unwrap_int() as jshort)
            }
            CPDType::CharType => {
                self.write_char(to_write.unwrap_int() as jchar) //todo assert zero
            }
            CPDType::IntType => {
                self.write_int(to_write.unwrap_int())
            }
            CPDType::LongType => {
                self.write_long(to_write.unwrap_long())
            }
            CPDType::FloatType => {
                self.write_float(to_write.unwrap_float())
            }
            CPDType::DoubleType => {
                self.write_double(to_write.unwrap_double())
            }
            CPDType::VoidType => {
                todo!()
            }
            CPDType::Class(_) |
            CPDType::Array { .. } => {
                self.write_object(to_write.unwrap_object().map(|obj|obj.cast().as_ptr()).unwrap_or(null_mut()))
            }
        }
    }
}

impl AccessorExt for FieldAccessor{

}


impl AccessorExt for ArrayAccessor {
}


impl AccessorExt for StaticField {

}
