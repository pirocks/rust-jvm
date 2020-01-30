use runtime_common::java_values::{JavaValue, Object};
use libffi::middle::Arg;
use libffi::middle::Type;
use crate::rust_jni::native_util::to_object;
use std::ffi::c_void;
use runtime_common::runtime_class::RuntimeClass;
use jni_bindings::jclass;
use std::sync::Arc;
use std::ops::Deref;

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
        JavaValue::Object(o) => match o {
            None => Arg::new(&(std::ptr::null() as *const Object)),
            Some(op) => {
                unsafe {
                    let object_ptr = to_object(op.into()) as *mut c_void;
                    let ref_box = Box::new(object_ptr);
                    //todo don;t forget to free later, and/or do this with lifetimes
                    Arg::new/*::<*mut c_void>*/(Box::leak(ref_box))
                }
            }
        },
        JavaValue::Top => panic!()
    }
}


pub fn runtime_class_to_native(runtime_class : Arc<RuntimeClass>) -> Arg{
    let boxed_arc = Box::new(runtime_class);
    let arc_pointer = Box::into_raw(boxed_arc);
    let pointer_ref = Box::leak(Box::new(arc_pointer));
    Arg::new(pointer_ref)
}


pub unsafe fn native_to_runtime_class(clazz: jclass) -> Arc<RuntimeClass>{
    let boxed_arc = Box::from_raw(clazz as *mut Arc<RuntimeClass>);
    boxed_arc.deref().clone()
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
        JavaValue::Object(_) => Type::pointer(),
        JavaValue::Top => panic!()
    }
}