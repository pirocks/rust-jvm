extern crate classfile;

use std::path::PathBuf;
use std::fs::File;
use classfile::classfile::parse_class_file;

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

