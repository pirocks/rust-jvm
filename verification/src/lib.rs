extern crate tempfile;
extern crate log;
extern crate simple_logger;

use std::sync::Arc;
use crate::verifier::class_is_type_safe;
use rust_jvm_common::loading::Loader;
use rust_jvm_common::classfile::Classfile;
use rust_jvm_common::unified_types::{UnifiedType, ClassWithLoader};
use crate::verifier::InternalFrame;
use rust_jvm_common::unified_types::ArrayType;
use rust_jvm_common::classnames::{get_referred_name, class_name};
use crate::verifier::Frame;
use crate::verifier::TypeSafetyError;


pub mod types;
pub mod verifier;


/**
We can only verify one class at a time, all needed classes need to be in jvm state as loading, including the class to verify.
*/
pub fn verify(to_verify: Arc<Classfile>,loader: Arc<dyn Loader + Send + Sync>) -> Result<(), TypeSafetyError> {
    match class_is_type_safe(unimplemented!(),&ClassWithLoader {
        class_name: class_name(&to_verify),
        loader,
    }) {
        Ok(_) => Result::Ok(()),
        Err(err) => {
            match err {
                TypeSafetyError::NotSafe(_) => unimplemented!(),
                TypeSafetyError::NeedToLoad(_) => unimplemented!(),
            }
        }
    }
}


#[derive(Debug)]
pub struct StackMap {
    pub offset: usize,
    pub map_frame: Frame,
}


pub fn init_frame(parameter_types: Vec<UnifiedType>, this_pointer: Option<UnifiedType>, max_locals: u16) -> InternalFrame {
    let mut locals = Vec::with_capacity(max_locals as usize);
    match this_pointer {
        None => {}//class is static etc.
        Some(t) => {
            locals_push_convert_type(&mut locals, t)
        }
    }
    for parameter_type in parameter_types {
        locals_push_convert_type(&mut locals, parameter_type)
    }
    InternalFrame { max_locals, locals, stack: Vec::new(), current_offset: 0 }
}

fn locals_push_convert_type(res: &mut Vec<UnifiedType>, type_: UnifiedType) -> () {
    match type_ {
        UnifiedType::ByteType => {
            res.push(UnifiedType::IntType);
        }
        UnifiedType::CharType => {
            res.push(UnifiedType::IntType);
        }
        UnifiedType::DoubleType => {
            res.push(UnifiedType::DoubleType);
            res.push(UnifiedType::TopType);
        }
        UnifiedType::FloatType => {
            res.push(UnifiedType::FloatType);
        }
        UnifiedType::IntType => {
            res.push(UnifiedType::IntType);
        }
        UnifiedType::LongType => {
            res.push(UnifiedType::LongType);
            res.push(UnifiedType::TopType);
        }
        UnifiedType::Class(r) => {
            assert_ne!(get_referred_name(&r.class_name).chars().nth(0).unwrap(), '[');
            res.push(UnifiedType::Class(r));
        }
        UnifiedType::ShortType => {
            res.push(UnifiedType::IntType);
        }
        UnifiedType::BooleanType => {
            res.push(UnifiedType::IntType);
        }
        UnifiedType::ArrayReferenceType(art) => {
            res.push(UnifiedType::ArrayReferenceType(
                ArrayType {
                    sub_type: Box::from(UnifiedType::ArrayReferenceType(art))
                }));
        }
        UnifiedType::VoidType => { panic!() }
        _ => { panic!("Case wasn't coverred with non-unified types") }
    }
}


pub struct VerifierContext{
    pub bootstrap_loader : Arc<dyn Loader + Send + Sync>
}

impl Clone for VerifierContext{
    fn clone(&self) -> Self {
        VerifierContext { bootstrap_loader: self.bootstrap_loader.clone() }
    }
}