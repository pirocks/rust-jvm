extern crate bimap;
extern crate classfile;
extern crate ntest_timeout;
extern crate timebomb;

use ntest_timeout::timeout;


use classfile::class_loading::{JVMState, class_entry_from_string, load_class};
use std::path::Path;
use classfile::classpath_indexing::index_class_path;
use classfile::verification::prolog_info_writer::BOOTSTRAP_LOADER_NAME;


#[test]
#[timeout(10000)]
pub fn can_verify_main() {
    let main_class_name = "Main".to_string();
    load_class_with_name(&main_class_name);
}

#[test]
//#[timeout(10000)]
pub fn can_verify_object() {
    let main_class_name = "java.lang.Object".to_string();
    load_class_with_name(&main_class_name);
}

#[test]
#[timeout(30000)]
pub fn can_verify_map() {
    let main_class_name = "java.util.Map".to_string();
    load_class_with_name(&main_class_name);
}

#[test]
#[timeout(30000)]
pub fn can_verify_exceptions() {
    let main_class_name = "java.lang.Throwable".to_string();
    load_class_with_name(&main_class_name);
    let main_class_name = "java.lang.Exception".to_string();
    load_class_with_name(&main_class_name);
    let main_class_name = "java.lang.IllegalArgumentException".to_string();
    load_class_with_name(&main_class_name);
}


fn load_class_with_name(main_class_name: &String) {
    let indexed_classpath = index_class_path(Path::new(&"resources/test/classpath_file".to_string()));
    let mut initial_jvm_state = JVMState {
        using_bootstrap_loader: true,
        loaders: vec![(BOOTSTRAP_LOADER_NAME.to_string(),BOOTSTRAP_LOADER.clone())].iter().cloned().collect(),//todo make correct
        indexed_classpath,
        using_prolog_verifier: false
    };
    use classfile::class_loading::BOOTSTRAP_LOADER;
    let main_class_entry = class_entry_from_string(&main_class_name, true);
    load_class(&mut initial_jvm_state, BOOTSTRAP_LOADER.clone(), main_class_entry, true);//todo add correct
}