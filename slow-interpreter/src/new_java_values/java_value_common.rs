use std::ptr::null_mut;
use jvmti_jni_bindings::{jboolean, jbyte, jchar, jdouble, jfloat, jint, jlong, jshort};
use rust_jvm_common::NativeJavaValue;
use crate::{JavaValue, NewJavaValue};

pub trait JavaValueCommon<'gc>{
    fn as_njv(&self) -> NewJavaValue<'gc, '_>;

    fn to_jv(&self) -> JavaValue<'gc> {
        todo!()
    }

    fn unwrap_bool_strict(&self) -> jboolean {
        match self.as_njv() {
            NewJavaValue::Boolean(bool) => bool,
            _ => {
                panic!()
            }
        }
    }

    fn unwrap_byte_strict(&self) -> jbyte {
        match self.as_njv() {
            NewJavaValue::Byte(byte) => {
                byte
            }
            _ => panic!()
        }
    }

    fn unwrap_char_strict(&self) -> jchar {
        match self.as_njv() {
            NewJavaValue::Char(char) => {
                char
            }
            _ => {
                panic!()
            }
        }
    }

    fn unwrap_short_strict(&self) -> jshort {
        todo!()
    }

    fn unwrap_int_strict(&self) -> jint {
        match self.as_njv() {
            NewJavaValue::Int(res) => res,
            _ => {
                panic!()
            }
        }
    }

    fn unwrap_int(&self) -> jint {
        match self.as_njv() {
            NewJavaValue::Int(int) => {
                int
            }
            NewJavaValue::Short(short) => {
                short as jint
            }
            NewJavaValue::Byte(byte) => {
                byte as jint
            }
            NewJavaValue::Boolean(bool) => {
                bool as jint
            }
            NewJavaValue::Char(char) => {
                char as jint
            }
            _ => panic!()
        }
    }

    fn unwrap_long_strict(&self) -> jlong {
        match self.as_njv() {
            NewJavaValue::Long(long) => {
                long
            }
            _ => panic!()
        }
    }

    fn unwrap_float_strict(&self) -> jfloat {
        match self.as_njv() {
            NewJavaValue::Float(float) => {
                float
            }
            _ => {
                panic!()
            }
        }
    }

    fn unwrap_double_strict(&self) -> jdouble {
        match self.as_njv() {
            NewJavaValue::Double(double) => {
                double
            }
            _ => {
                panic!()
            }
        }
    }

    fn to_native(&self) -> NativeJavaValue<'gc> {
        let mut all_zero = NativeJavaValue { as_u64: 0 };
        match self.as_njv() {
            NewJavaValue::Long(long) => {
                all_zero.long = long;
            }
            NewJavaValue::Int(int) => {
                all_zero.int = int;
            }
            NewJavaValue::Short(short) => {
                all_zero.short = short;
            }
            NewJavaValue::Byte(byte) => {
                all_zero.byte = byte;
            }
            NewJavaValue::Boolean(bool) => {
                all_zero.boolean = bool;
            }
            NewJavaValue::Char(char) => {
                all_zero.char = char;
            }
            NewJavaValue::Float(float) => {
                all_zero.float = float;
            }
            NewJavaValue::Double(double) => {
                all_zero.double = double;
            }
            NewJavaValue::Null => {
                all_zero.object = null_mut();
            }
            NewJavaValue::UnAllocObject(_) => {
                todo!()
            }
            NewJavaValue::AllocObject(obj) => {
                all_zero.object = obj.ptr().as_ptr();
            }
            NewJavaValue::Top => {
                all_zero.as_u64 = 0xdddd_dddd_dddd_dddd;
            }
        }
        all_zero
    }
}

