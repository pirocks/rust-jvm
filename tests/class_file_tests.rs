extern crate bimap;
extern crate classfile;

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::io;
use std::path::PathBuf;

use classfile::class_loading::JVMClassesState;
use classfile::classfile::parse_class_file;
use classfile::verification::prolog_info_defs::{ExtraDescriptors, gen_prolog, PrologGenContext};

#[test]
pub fn basic_class_file_parse() {
    use classfile::classfile::parsing_util::ParsingContext;
    let mut test_resources_path = get_test_resources();
    test_resources_path.push("Main.class");

    let mut p = ParsingContext { f : File::open(test_resources_path.as_os_str()).unwrap() };
    let _parsed = parse_class_file(&mut p);
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
    use classfile::classfile::parsing_util::ParsingContext;
    let mut test_resources_path = get_test_resources();
    test_resources_path.push("Main.class");

    let mut p = ParsingContext { f : File::open(test_resources_path.as_os_str()).unwrap() };
    let parsed = parse_class_file(&mut p);
    let mut class_files = Vec::new();
    class_files.push(parsed);
    let mut prolog_context = PrologGenContext {
        to_verify: class_files,
        state: &JVMClassesState {
            indexed_classpath: HashMap::new(),
            loading_in_progress: HashSet::new(),
            using_bootstrap_loader: true,
            bootstrap_loaded_classes: HashMap::new(),
            partial_load: HashSet::new(),
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