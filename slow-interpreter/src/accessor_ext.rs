use std::ptr::{NonNull, null_mut};

use jvmti_jni_bindings::{jboolean, jbyte, jchar, jobject};
use runtime_class_stuff::array_layout::ArrayAccessor;
use runtime_class_stuff::object_layout::FieldAccessor;
use rust_jvm_common::compressed_classfile::compressed_types::CPDType;

use crate::{JavaValueCommon, JVMState, NewJavaValue, NewJavaValueHandle};
use crate::interpreter::real_interpreter_state::InterpreterJavaValue;

pub trait FieldAccessorExt {
    fn write_njv<'gc, 'l>(self, njv: NewJavaValue<'gc, 'l>, expected_type: CPDType);
    fn read_njv<'gc>(self, jvm: &'gc JVMState<'gc>, expected_type: CPDType) -> NewJavaValueHandle<'gc>;
    fn read_interpreter_jv(self, expected_type: CPDType) -> InterpreterJavaValue;
    fn write_interpreter_jv(self, to_write: InterpreterJavaValue, expected_type: CPDType);
}

impl FieldAccessorExt for FieldAccessor {
    fn write_njv<'gc, 'l>(self, njv: NewJavaValue<'gc, 'l>, expected_type: CPDType) {
        match expected_type {
            CPDType::BooleanType => {
                self.write_boolean(njv.unwrap_int() as jboolean)//todo add an assert that there can be no other vals set?
            }
            CPDType::ByteType => {
                todo!()
            }
            CPDType::ShortType => {
                todo!()
            }
            CPDType::CharType => {
                todo!()
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
                todo!()
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

    fn read_njv<'gc>(self, jvm: &'gc JVMState<'gc>, expected_type: CPDType) -> NewJavaValueHandle<'gc> {
        match expected_type {
            CPDType::BooleanType => {
                todo!()
            }
            CPDType::ByteType => {
                todo!()
            }
            CPDType::ShortType => {
                todo!()
            }
            CPDType::CharType => {
                todo!()
            }
            CPDType::IntType => {
                NewJavaValueHandle::Int(self.read_int())
            }
            CPDType::LongType => {
                NewJavaValueHandle::Long(self.read_long())
            }
            CPDType::FloatType => {
                todo!()
            }
            CPDType::DoubleType => {
                todo!()
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

    fn read_interpreter_jv(self, expected_type: CPDType) -> InterpreterJavaValue {
        todo!()
    }

    fn write_interpreter_jv(self, to_write: InterpreterJavaValue, expected_type: CPDType) {
        todo!()
    }
}

pub trait ArrayAccessorExt {
    fn write_njv<'gc, 'l>(self, njv: NewJavaValue<'gc, 'l>, expected_type: CPDType);
    fn read_njv<'gc>(self, jvm: &'gc JVMState<'gc>, expected_type: CPDType) -> NewJavaValueHandle<'gc>;
    fn read_interpreter_jv(self, expected_type: CPDType) -> InterpreterJavaValue;
    fn write_interpreter_jv(self, to_write: InterpreterJavaValue, expected_type: CPDType);
}


impl ArrayAccessorExt for ArrayAccessor {
    fn write_njv<'gc, 'l>(self, njv: NewJavaValue<'gc, 'l>, expected_type: CPDType) {
        match expected_type {
            CPDType::BooleanType => {
                self.write_boolean(njv.unwrap_int() as jboolean)//todo add an assert that there can be no other vals set?
            }
            CPDType::ByteType => {
                self.write_byte(njv.unwrap_int() as jbyte)//todo add an assert that there can be no other vals set?
            }
            CPDType::ShortType => {
                todo!()
            }
            CPDType::CharType => {
                self.write_char(njv.unwrap_int() as jchar)//todo add an assert that conversion happens well
            }
            CPDType::IntType => {
                self.write_int(njv.unwrap_int());
            }
            CPDType::LongType => {
                todo!()
            }
            CPDType::FloatType => {
                todo!()
            }
            CPDType::DoubleType => {
                todo!()
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

    fn read_njv<'gc>(self, jvm: &'gc JVMState<'gc>, expected_type: CPDType) -> NewJavaValueHandle<'gc> {
        match expected_type {
            CPDType::BooleanType => {
                todo!()
            }
            CPDType::ByteType => {
                todo!()
            }
            CPDType::ShortType => {
                todo!()
            }
            CPDType::CharType => {
                NewJavaValueHandle::Char(self.read_char())
            }
            CPDType::IntType => {
                NewJavaValueHandle::Int(self.read_int())
            }
            CPDType::LongType => {
                todo!()
            }
            CPDType::FloatType => {
                todo!()
            }
            CPDType::DoubleType => {
                todo!()
            }
            CPDType::VoidType => {
                todo!()
            }
            CPDType::Class(_) |
            CPDType::Array { .. } => {
                todo!()
            }
        }
    }

    fn read_interpreter_jv(self, expected_type: CPDType) -> InterpreterJavaValue {
        match expected_type {
            CPDType::BooleanType => {
                todo!()
            }
            CPDType::ByteType => {
                todo!()
            }
            CPDType::ShortType => {
                todo!()
            }
            CPDType::CharType => {
                InterpreterJavaValue::Int(self.read_char() as i32)
            }
            CPDType::IntType => {
                todo!()
            }
            CPDType::LongType => {
                todo!()
            }
            CPDType::FloatType => {
                todo!()
            }
            CPDType::DoubleType => {
                todo!()
            }
            CPDType::VoidType => {
                todo!()
            }
            CPDType::Class(_) |
            CPDType::Array { .. } => {
                InterpreterJavaValue::Object(NonNull::new(self.read_object()).map(|nonnull|nonnull.cast()))
            }
        }
    }

    fn write_interpreter_jv(self, to_write: InterpreterJavaValue, expected_type: CPDType) {
        match expected_type {
            CPDType::BooleanType => {
                todo!()
            }
            CPDType::ByteType => {
                todo!()
            }
            CPDType::ShortType => {
                todo!()
            }
            CPDType::CharType => {
                self.write_char(to_write.unwrap_int() as u16)//todo assert no leftover bytes?
            }
            CPDType::IntType => {
                todo!()
            }
            CPDType::LongType => {
                todo!()
            }
            CPDType::FloatType => {
                todo!()
            }
            CPDType::DoubleType => {
                todo!()
            }
            CPDType::VoidType => {
                todo!()
            }
            CPDType::Class(_) |
            CPDType::Array { .. } => {
                self.write_object(to_write.unwrap_object().map(|nonnull|nonnull.cast().as_ptr()).unwrap_or(null_mut()))
            }
        }

    }
}
