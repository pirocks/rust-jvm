extern crate ntest_timeout;
extern crate timebomb;
extern crate rust_jvm_common;

use ntest_timeout::timeout;
use rust_jvm_common::test_utils::get_test_resources;
use std::collections::HashMap;
use verification::verifier::TypeSafetyError;
use std::sync::Arc;
use std::path::Path;
use classfile_parser::parse_class_file;
use std::fs::File;
use loading::BootstrapLoader;
use std::sync::RwLock;
use loading::Classpath;
use rust_jvm_common::loading::LoaderName;
use verification::verify;
use verification::VerifierContext;
use rust_jvm_common::classnames::class_name;

#[test]
#[timeout(10000)]
pub fn can_verify_main() {
    let main_class_name = "Main".to_string();
    verify_class_with_name(&main_class_name).unwrap();
}

#[test]
#[timeout(10000)]
pub fn can_verify_float_double_arithmetic() {
    let main_class_name = "FloatDoubleArithmetic".to_string();
    verify_class_with_name(&main_class_name).unwrap();
}

#[test]
#[timeout(10000)]
pub fn can_verify_with_main() {
    let main_class_name = "WithMain".to_string();
    verify_class_with_name(&main_class_name).unwrap();
}


#[test]
#[timeout(10000)]
pub fn can_verify_object() {
    let main_class_name = "java/lang/Object".to_string();
    verify_class_with_name(&main_class_name).unwrap();
}


#[test]
//#[timeout(30000)]
pub fn can_verify_map() {
    let main_class_name = "java/util/Map".to_string();
    verify_class_with_name(&main_class_name).unwrap();
}

#[test]
//#[timeout(30000)]
pub fn can_verify_exceptions() {
    let main_class_name = "java/lang/Throwable".to_string();
    verify_class_with_name(&main_class_name).unwrap();
    let main_class_name = "java/lang/Exception".to_string();
    verify_class_with_name(&main_class_name).unwrap();
    let main_class_name = "java/lang/IllegalArgumentException".to_string();
    verify_class_with_name(&main_class_name).unwrap();
}

#[test]
//#[timeout(10000)]
pub fn can_verify_hash_map() {
    let main_class_name = "java/util/HashMap".to_string();
    verify_class_with_name(&main_class_name).unwrap();
}


fn verify_class_with_name(main_class_name: &String) -> Result<(),TypeSafetyError>{
    let mut resources = get_test_resources();
    resources.push(format!("{}.class",main_class_name));
    verify_impl(resources.as_path(),get_test_resources().as_path())
}

fn verify_impl(path_of_class: &Path, class_path : &Path) -> Result<(), TypeSafetyError> {
    let loader = BootstrapLoader {
        loaded: RwLock::new(HashMap::new()),
        parsed: RwLock::new(HashMap::new()),
        name: RwLock::new(LoaderName::BootstrapLoader),
        classpath: Classpath { classpath_base: vec![class_path.to_path_buf().into_boxed_path()] }
    };
    let file = File::open(path_of_class).unwrap();
    let bootstrap_loader = Arc::new(loader);
    let classfile = parse_class_file((&file).try_clone().unwrap(), bootstrap_loader.clone());
    bootstrap_loader.parsed.write().unwrap().insert(class_name(&classfile),classfile.clone());


    match verify(&VerifierContext{ bootstrap_loader:bootstrap_loader.clone() }, classfile.clone(),bootstrap_loader.clone()){
        Ok(_) => Result::Ok(()),
        Err(err) => {
            match err {
                TypeSafetyError::NotSafe(s) => {dbg!(s);assert!(false);panic!()},
                TypeSafetyError::NeedToLoad(ntl) => {
                    dbg!(ntl);
                    assert!(false);
                    panic!();
                }
            }
        },
    }
}