use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::io;
use std::io::Lines;
use std::process::{Child, ChildStdin, ChildStdout, Stdio};
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

use log::trace;
use regex::Regex;
use tempfile::NamedTempFile;

use std::sync::Arc;
use crate::verification::verifier::{class_is_type_safe, TypeSafetyResult, PrologClass};
use rust_jvm_common::loading::{ClassEntry, Loader, JVMState};
use rust_jvm_common::classfile::Classfile;
use rust_jvm_common::classnames::class_name_legacy;
use classfile_parser::classfile::parse_class_file;
use crate::verification::prolog::prolog_verify;

/**
We can only verify one class at a time, all needed classes need to be in jvm state as loading, including the class to verify.
*/
pub fn verify(to_verify: &HashMap<ClassEntry, Arc<Classfile>>, jvm_state: &mut JVMState, loader: Arc<Loader>) -> TypeSafetyResult {
    if jvm_state.using_prolog_verifier {
        prolog_verify(jvm_state, to_verify);
        unimplemented!()
    } else {
        to_verify.iter().for_each(|(x,_)|{
            trace!("Attempting to verify: {} ",x);
        });
        use crate::verification::verifier::merge_type_safety_results;
        let verification_results: Vec<TypeSafetyResult> = to_verify.iter().map(|(entry, loaded)| {
            let current_class = PrologClass {
                class: loaded.clone(),
                loader:loader.clone(),
            };
            class_is_type_safe(&current_class)
        }).collect();
        dbg!(&verification_results);
        merge_type_safety_results(verification_results.into_boxed_slice())
    }
}

pub mod prolog;
pub mod types;
pub mod instruction_outputer;
pub mod verifier;