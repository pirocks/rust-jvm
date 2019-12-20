use classfile_parser::parse_class_file;
use std::fs::File;
use rust_jvm_common::test_utils::get_test_resources;

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