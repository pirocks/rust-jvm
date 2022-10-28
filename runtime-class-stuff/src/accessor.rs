use jvmti_jni_bindings::{jboolean, jbyte, jchar, jdouble, jfloat, jint, jlong, jobject, jshort};
use rust_jvm_common::compressed_classfile::compressed_types::CPDType;

pub trait Accessor {
    fn expected_type(&self) -> CPDType;
    fn read_impl<T>(&self) -> T;
    fn read_boolean(&self) -> jboolean {
        assert_eq!(CPDType::BooleanType, self.expected_type());
        self.read_impl()
    }

    fn read_byte(&self) -> jbyte {
        assert_eq!(CPDType::ByteType, self.expected_type());
        self.read_impl()
    }

    fn read_short(&self) -> jshort {
        assert_eq!(CPDType::ShortType, self.expected_type());
        self.read_impl()
    }

    fn read_char(&self) -> jchar {
        assert_eq!(CPDType::CharType, self.expected_type());
        self.read_impl()
    }

    fn read_int(&self) -> jint {
        assert_eq!(CPDType::IntType, self.expected_type());
        self.read_impl()
    }

    fn read_float(&self) -> jfloat {
        assert_eq!(CPDType::FloatType, self.expected_type());
        self.read_impl()
    }

    fn read_long(&self) -> jlong {
        assert_eq!(CPDType::LongType, self.expected_type());
        self.read_impl()
    }

    fn read_double(&self) -> jdouble {
        assert_eq!(CPDType::FloatType, self.expected_type());
        self.read_impl()
    }

    fn read_object(&self) -> jobject {
        assert!(&self.expected_type().try_unwrap_ref_type().is_some());
        self.read_impl()
    }
    fn write_impl<T>(&self, to_write: T);
    fn write_boolean(&self, to_write: jboolean) {
        assert_eq!(CPDType::BooleanType, self.expected_type());
        self.write_impl(to_write)
    }

    fn write_byte(&self, to_write: jbyte) {
        assert_eq!(CPDType::ByteType, self.expected_type());
        self.write_impl(to_write)
    }

    fn write_short(&self, to_write: jshort) {
        assert_eq!(CPDType::ShortType, self.expected_type());
        self.write_impl(to_write)
    }

    fn write_char(&self, to_write: jchar) {
        assert_eq!(CPDType::CharType, self.expected_type());
        self.write_impl(to_write)
    }

    fn write_int(&self, to_write: jint) {
        assert_eq!(CPDType::IntType, self.expected_type());
        self.write_impl(to_write)
    }

    fn write_float(&self, to_write: jfloat) {
        assert_eq!(CPDType::FloatType, self.expected_type());
        self.write_impl(to_write)
    }

    fn write_long(&self, to_write: jlong) {
        assert_eq!(CPDType::LongType, self.expected_type());
        self.write_impl(to_write)
    }

    fn write_double(&self, to_write: jdouble) {
        assert_eq!(CPDType::DoubleType, self.expected_type());
        self.write_impl(to_write)
    }

    fn write_object(&self, to_write: jobject) {
        assert!(self.expected_type().try_unwrap_ref_type().is_some());
        self.write_impl(to_write)
    }
}
