extern crate tempfile;
extern crate log;
extern crate simple_logger;


use std::collections::HashMap;

use log::trace;

use std::sync::Arc;
use crate::verifier::{class_is_type_safe, PrologClass};
use rust_jvm_common::loading::{ClassEntry, Loader, JVMState};
use rust_jvm_common::classfile::Classfile;
use crate::prolog::prolog_verify;
use crate::verifier::TypeSafetyError;

/**
We can only verify one class at a time, all needed classes need to be in jvm state as loading, including the class to verify.
*/
pub fn verify(to_verify: &HashMap<ClassEntry, Arc<Classfile>>, jvm_state: &mut JVMState, loader: Arc<Loader>) -> Result<(),TypeSafetyError> {
    if jvm_state.using_prolog_verifier {
        prolog_verify(jvm_state, to_verify);
        unimplemented!()
    } else {
        to_verify.iter().for_each(|(x,_)|{
            trace!("Attempting to verify: {} ",x);
        });
        let verification_results: Result<Vec<_>,_> = to_verify.iter().map(|(_entry, loaded)| {
            let current_class = PrologClass {
                class: loaded.clone(),
                loader:loader.clone(),
            };
            class_is_type_safe(&current_class)
        }).collect();
        verification_results?;
        Result::Ok(())
    }
}

pub mod prolog;
pub mod types;
pub mod instruction_outputer;
pub mod verifier;