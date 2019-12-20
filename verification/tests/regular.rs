extern crate ntest_timeout;
extern crate timebomb;
extern crate rust_jvm_common;

use ntest_timeout::timeout;
use rust_jvm_common::loading::{class_entry_from_string, BOOTSTRAP_LOADER_NAME, BOOTSTRAP_LOADER,JVMState};
use rust_jvm_common::test_utils::get_test_resources;
use verification::verification::verify;
use classfile_parser::classfile::parse_class_file;
use std::fs::File;
use std::collections::HashMap;
use verification::verification::verifier::TypeSafetyResult;

#[test]
//#[timeout(10000)]
pub fn can_verify_main() {
    let main_class_name = "Main".to_string();
    verify_class_with_name(&main_class_name);
}

#[test]
//#[timeout(10000)]
pub fn can_verify_object() {
    let main_class_name = "java/lang/Object".to_string();
    verify_class_with_name(&main_class_name);
}


#[test]
#[timeout(30000)]
pub fn can_verify_map() {
    let main_class_name = "java/util/Map".to_string();
    verify_class_with_name(&main_class_name);
}

#[test]
#[timeout(30000)]
pub fn can_verify_exceptions() {
    let main_class_name = "java/lang/Throwable".to_string();
    verify_class_with_name(&main_class_name);
    let main_class_name = "java/lang/Exception".to_string();
    verify_class_with_name(&main_class_name);
    let main_class_name = "java/lang/IllegalArgumentException".to_string();
    verify_class_with_name(&main_class_name);
}



fn verify_class_with_name(main_class_name: &String) -> TypeSafetyResult{
    let mut resources = get_test_resources();
    resources.push(format!("{}.class",main_class_name));
    let classfile = parse_class_file(File::open(resources.as_path()).unwrap());
    let mut to_verify = HashMap::new();
    to_verify.insert(class_entry_from_string(main_class_name,false), classfile);
    let mut loaders = HashMap::new();
    loaders.insert(BOOTSTRAP_LOADER_NAME.to_string(),BOOTSTRAP_LOADER.clone());
    verify(&to_verify, &mut JVMState{
        using_bootstrap_loader: true,
        loaders: loaders,
        indexed_classpath: Default::default(),
        using_prolog_verifier: false
    }, BOOTSTRAP_LOADER.clone())
}