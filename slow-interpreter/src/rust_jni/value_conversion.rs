use runtime_common::java_values::{JavaValue, Object};
use libffi::middle::Arg;
use libffi::middle::Type;
use crate::rust_jni::native_util::to_object;
use std::ffi::c_void;
use runtime_common::runtime_class::RuntimeClass;
use jni_bindings::jclass;
use std::sync::Arc;
use std::ops::Deref;
use rust_jvm_common::unified_types::ParsedType;

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


pub fn to_native_type(t: &ParsedType) -> Type {

//    pub type jint = i32;
//    pub type jlong = i64;
//    pub type jbyte = i8;
//    pub type jboolean = u8;
//    pub type jchar = u16;
//    pub type jshort = i16;
//    pub type jfloat = f32;
//    pub type jdouble = f64;
//    pub type jsize = jint;

    match t {
        ParsedType::ByteType => Type::i8(),
        ParsedType::CharType => Type::u16(),
        ParsedType::DoubleType => Type::f64(),
        ParsedType::FloatType => Type::f32(),
        ParsedType::IntType => Type::i32(),
        ParsedType::LongType => Type::i64(),
        ParsedType::Class(_) => Type::pointer(),
        ParsedType::ShortType => Type::i16(),
        ParsedType::BooleanType => Type::u8(),
        ParsedType::ArrayReferenceType(_) => Type::pointer(),
        ParsedType::VoidType => unimplemented!(),
        ParsedType::TopType => unimplemented!(),
        ParsedType::NullType => unimplemented!(),
        ParsedType::Uninitialized(_) => unimplemented!(),
        ParsedType::UninitializedThis => unimplemented!(),
        ParsedType::UninitializedThisOrClass(_) => unimplemented!(),
    }
}


pub fn to_native(j: JavaValue, t: &ParsedType) -> Arg {
    match t {
        ParsedType::ByteType => {
            Arg::new(Box::leak(Box::new(j.unwrap_int() as i8)))//todo free after call
        },
        ParsedType::CharType => {
            Arg::new(Box::leak(Box::new(j.unwrap_int() as u16)))//todo free after call
        },
        ParsedType::DoubleType => {
            Arg::new(Box::leak(Box::new(j.unwrap_double())))//todo free after call
        },
        ParsedType::FloatType => {
            Arg::new(Box::leak(Box::new(j.unwrap_float())))//todo free after call
        },
        ParsedType::IntType => {
            Arg::new(Box::leak(Box::new(j.unwrap_int())))//todo free after call
        },
        ParsedType::LongType => {
            Arg::new(Box::leak(Box::new(j.unwrap_long())))//todo free after call
        },
        ParsedType::Class(_) => {
            match j.unwrap_object(){
                None => Arg::new(&(std::ptr::null() as *const Object)),
                Some(op) => {
                    unsafe {
                        let object_ptr = to_object(op.into()) as *mut c_void;
                        let ref_box = Box::new(object_ptr);
                        //todo don;t forget to free later, and/or do this with lifetimes
                        Arg::new/*::<*mut c_void>*/(Box::leak(ref_box))
                    }
                }
            }
        },
        ParsedType::ShortType => {
            Arg::new(Box::leak(Box::new(j.unwrap_int() as i16)))//todo free after call
        },
        ParsedType::BooleanType => {
            Arg::new(Box::leak(Box::new(j.unwrap_int() as u8)))//todo free after call
        },
        ParsedType::ArrayReferenceType(_) => {
            //todo dup
            match j.unwrap_object(){
                None => Arg::new(&(std::ptr::null() as *const Object)),
                Some(op) => {
                    unsafe {
                        let object_ptr = to_object(op.into()) as *mut c_void;
                        let ref_box = Box::new(object_ptr);
                        //todo don;t forget to free later, and/or do this with lifetimes
                        Arg::new/*::<*mut c_void>*/(Box::leak(ref_box))
                    }
                }
            }
        },
        ParsedType::VoidType => panic!(),
        ParsedType::TopType => panic!(),
        ParsedType::NullType => panic!(),
        ParsedType::Uninitialized(_) => panic!(),
        ParsedType::UninitializedThis => panic!(),
        ParsedType::UninitializedThisOrClass(_) => panic!(),
    }
    /*match j {
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
                    Arg::new*//*::<*mut c_void>*//*(Box::leak(ref_box))
                }
            }
        },
        JavaValue::Top => panic!()
    }*/
}
