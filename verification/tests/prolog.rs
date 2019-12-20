use std::io;
use classfile_parser::classfile::parse_class_file;
use std::fs::File;
use rust_jvm_common::test_utils::get_test_resources;
use verification::verification::prolog::prolog_info_writer::{PrologGenContext, ExtraDescriptors, gen_prolog};
use rust_jvm_common::loading::JVMState;
use std::collections::HashMap;
use std::io::{BufWriter, Write};

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