use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;

fn parse_classpath_file(path : &Path) -> Vec<Box<String>>{
    let mut f = File::open(path).expect("Error opening classpath file");
    let mut file_contents = String::new();
    f.read_to_string(&mut file_contents).expect("Error reading classpath file");
    file_contents.lines().flat_map(|line|{
        line.split(":")
    }).map(|s| {
        Box::new(s.to_string())
    }).collect()
}

#[test]
fn test_parse_classpath_file(){
    get_test_resources().push("classpath_file")
}

fn get_test_resources() -> PathBuf {
    let mut test_resources_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_resources_path.push("resources/test");
    test_resources_path
}