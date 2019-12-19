extern crate classfile_parser;
extern crate verification;
extern crate rust_jvm_common;

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::io;
use std::path::PathBuf;
use classfile_parser::classfile::parse_class_file;
use verification::verification::prolog_info_writer::{PrologGenContext, ExtraDescriptors, gen_prolog};
use rust_jvm_common::loading::JVMState;


#[test]
pub fn basic_class_file_parse() {
    let mut test_resources_path = get_test_resources();
    test_resources_path.push("Main.class");

    let _parsed = parse_class_file(File::open(test_resources_path.as_os_str()).unwrap());
//    dbg!(parsed);
    //todo asserts
//    assert!(false);
    return;
}

fn get_test_resources() -> PathBuf {
    let mut test_resources_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_resources_path.push("resources/test");
    test_resources_path
}

#[test]
pub fn basic_class_file_prolog_output() -> Result<(),io::Error>{
    let mut test_resources_path = get_test_resources();
    test_resources_path.push("Main.class");

    let parsed = parse_class_file(File::open(test_resources_path.as_os_str()).unwrap());
    let mut class_files = Vec::new();
    class_files.push(parsed);
    let mut prolog_context = PrologGenContext {
        to_verify: class_files,
        state: &JVMState {
            indexed_classpath: HashMap::new(),
            using_bootstrap_loader: true,
            loaders: Default::default(),//todo add correct field
            using_prolog_verifier: false
        },
        extra: ExtraDescriptors {
            extra_method_descriptors: Vec::new(),
            extra_field_descriptors: Vec::new(),
        },
    };
    let mut writer = BufWriter::new(std::io::stdout());

    gen_prolog(&mut prolog_context, &mut writer)?;
    writer.flush()?;
//    assert!(false);
    return Ok(());
}