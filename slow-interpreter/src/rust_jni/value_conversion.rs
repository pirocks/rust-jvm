use runtime_common::java_values::{JavaValue, Object};
use libffi::middle::Arg;
use libffi::middle::Type;
use crate::rust_jni::native_util::to_object;
use std::ffi::c_void;

pub fn to_native(j: JavaValue) -> Arg {
    match j {
        //todo suspect primitive types don't work
        JavaValue::Long(l) => Arg::new(&l),
        JavaValue::Int(i) => Arg::new(&i),
        JavaValue::Short(s) => Arg::new(&s),
        JavaValue::Byte(b) => Arg::new(&b),
        JavaValue::Boolean(b) => Arg::new(&b),
        JavaValue::Char(c) => Arg::new(&c),
        JavaValue::Float(f) => Arg::new(&f),
        JavaValue::Double(d) => Arg::new(&d),
        JavaValue::Array(_) => unimplemented!(),
        JavaValue::Object(o) => match o {
            None => Arg::new(&(std::ptr::null() as *const Object)),
            Some(op) => {
                unsafe {
                    let object_ptr = to_object(op.object) as *mut c_void;
                    dbg!(object_ptr);
                    Arg::new::<*mut c_void>(&object_ptr)
                }
            }
        },
        JavaValue::Top => panic!()
    }
}


pub fn to_native_type(j: JavaValue) -> Type {

//    pub type jint = i32;
//    pub type jlong = i64;
//    pub type jbyte = i8;
//    pub type jboolean = u8;
//    pub type jchar = u16;
//    pub type jshort = i16;
//    pub type jfloat = f32;
//    pub type jdouble = f64;
//    pub type jsize = jint;
//
//    pub enum _jobject {}
//    pub type jobject = *mut _jobject;
//    pub type jclass = jobject;
//    pub type jthrowable = jobject;
//    pub type jstring = jobject;
//    pub type jarray = jobject;
//    pub type jbooleanArray = jarray;
//    pub type jbyteArray = jarray;
//    pub type jcharArray = jarray;
//    pub type jshortArray = jarray;
//    pub type jintArray = jarray;
//    pub type jlongArray = jarray;
//    pub type jfloatArray = jarray;
//    pub type jdoubleArray = jarray;
//    pub type jobjectArray = jarray;
//    pub type jweak = jobject;

    match j {
        JavaValue::Long(_) => Type::i64(),
        JavaValue::Int(_) => Type::i32(),
        JavaValue::Short(_) => Type::i16(),
        JavaValue::Byte(_) => Type::i8(),
        JavaValue::Boolean(_) => Type::u8(),
        JavaValue::Char(_) => Type::i16(),
        JavaValue::Float(_) => Type::f32(),
        JavaValue::Double(_) => Type::f64(),
        JavaValue::Array(_) => unimplemented!(),
        JavaValue::Object(_) => Type::pointer(),
        JavaValue::Top => panic!()
    }
}