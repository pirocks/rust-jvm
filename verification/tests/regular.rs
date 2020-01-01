extern crate ntest_timeout;
extern crate timebomb;
extern crate rust_jvm_common;

use ntest_timeout::timeout;
use rust_jvm_common::test_utils::get_test_resources;
use std::collections::HashMap;
use verification::verifier::TypeSafetyError;
use std::sync::Arc;
use rust_jvm_common::classfile::Classfile;
use std::path::Path;
use rust_jvm_common::classnames::ClassName;

#[test]
//#[timeout(10000)]
pub fn can_verify_main() {
    let main_class_name = "Main".to_string();
    verify_class_with_name(&main_class_name).unwrap();
}

#[test]
//#[timeout(10000)]
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



fn verify_class_with_name(main_class_name: &String) -> Result<(),TypeSafetyError>{
    let mut resources = get_test_resources();
    resources.push(format!("{}.class",main_class_name));

    let mut to_verify = HashMap::new();
    verify_impl(main_class_name, &mut to_verify, resources.as_path())
}

fn verify_impl(
    _main_class_name: &String,
    _to_verify: &mut HashMap<ClassName, Arc<Classfile>>,
    _path: &Path
) -> Result<(), TypeSafetyError> {
//    let file = File::open(path).unwrap();
//    let classfile = parse_class_file((&file).try_clone().unwrap(), BOOTSTRAP_LOADER.clone());
//    to_verify.insert(class_entry_from_string(main_class_name, false), classfile);
//    let mut loaders = HashMap::new();
//    loaders.insert(BOOTSTRAP_LOADER_NAME.to_string(), BOOTSTRAP_LOADER.clone());
    unimplemented!()
//    match verify(&to_verify, &mut JVMState {
//        using_bootstrap_loader: true,
//        loaders,
//        indexed_classpath: Default::default(),
//        using_prolog_verifier: false
//    }, BOOTSTRAP_LOADER.clone()){
//        Ok(_) => Result::Ok(()),
//        Err(err) => {
//            match err {
//                TypeSafetyError::NotSafe(s) => {dbg!(s);assert!(false);panic!()},
//                TypeSafetyError::NeedToLoad(ntl) => {
//                    for c in ntl {
//                        let mut resources = get_test_resources();
//                        resources.push(format!("{}.class",get_referred_name(&c)));
//                        let file = File::open(resources.as_path()).unwrap();
//                        let classfile = parse_class_file(file,BOOTSTRAP_LOADER.clone());
//                        to_verify.insert(class_entry_from_string(&get_referred_name(&c), false), classfile.clone());
//                        let loader: Arc<dyn Loader + Send + Sync> = BOOTSTRAP_LOADER.clone();
//                        loader.loading.write().unwrap().insert(class_entry_from_string(&get_referred_name(&c), false), classfile.clone());
//                    }
//                    verify_impl(main_class_name,to_verify,path)
//                }
//            }
//        },
//    }
}