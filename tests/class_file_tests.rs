extern crate classfile;

use std::path::PathBuf;
use std::fs::File;
use classfile::classfile::parse_class_file;
use classfile::verification::PrologGenContext;
use std::io::{BufWriter, Write};
use std::io;

#[test]
pub fn basic_class_file_parse() {
    use classfile::classfile::parsing_util::ParsingContext;
    let mut test_resources_path = get_test_resources();
    test_resources_path.push("Main.class");

    let mut p = ParsingContext { f : File::open(test_resources_path.as_os_str()).unwrap() };
    let parsed = parse_class_file(&mut p);
    dbg!(parsed);
    //todo asserts
    assert!(false);
    return;
}

fn get_test_resources() -> PathBuf {
    let mut test_resources_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_resources_path.push("resources/test");
    test_resources_path
}

extern crate bimap;

#[test]
pub fn basic_class_file_prolog_output() -> Result<(),io::Error>{
    use bimap::BiMap;
    use classfile::classfile::parsing_util::ParsingContext;
    let mut test_resources_path = get_test_resources();
    test_resources_path.push("Main.class");

    let mut p = ParsingContext { f : File::open(test_resources_path.as_os_str()).unwrap() };
    let parsed = parse_class_file(&mut p);
    let mut class_files = Vec::new();
    class_files.push(parsed);
    let prolog_context = PrologGenContext{class_files,name_to_classfile:(BiMap::new()) };
    use classfile::verification::gen_prolog;
    let mut writer = BufWriter::new(std::io::stdout());

    gen_prolog(&prolog_context, &mut writer)?;
    writer.flush()?;
//    assert!(false);
    return Ok(());
}