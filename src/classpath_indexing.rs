use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;
use class_loading::{class_entry_from_string, ClassEntry};
use std::collections::{HashSet, HashMap};

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

pub fn index_class_path(classfile_path : &Path) -> HashMap<ClassEntry,Box<Path>>{
    let entries= parse_classpath_file(classfile_path);
    entries.iter().map(|s|{
        let mut path_buf = PathBuf::new();
        path_buf.push(s.as_str());
        let path = path_buf.into_boxed_path();
        //todo read entry form classfile, b/c its easier that way.
        (class_entry_from_string(s),path)
    }).collect()
}

#[test]
fn test_parse_classpath_file(){
    let mut classpath_file = get_test_resources();
    classpath_file.push("classpath_file");
    let res = parse_classpath_file(classpath_file.as_path());
    dbg!(&res);
    assert!(res.contains(&Box::new("/home/francis/rust-jvm/resources/test/Main.class".to_string())));
    assert!(res.contains(&Box::new("/home/francis/unzipped-java/java.base/sun/text/ComposedCharIter.class".to_string())));
    assert!(res.contains(&Box::new( "/home/francis/unzipped-java/java.base/sun/text/normalizer/NormalizerBase$NFCMode.class".to_string())))
}

fn get_test_resources() -> PathBuf {
    let mut test_resources_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_resources_path.push("resources/test");
    test_resources_path
}