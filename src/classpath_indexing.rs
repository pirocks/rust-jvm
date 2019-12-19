use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

extern crate walkdir;
extern crate pathdiff;
use self::walkdir::{WalkDir};

use rust_jvm_common::loading::ClassEntry;

fn parse_classpath_file(path : &Path) -> Vec<Box<String>>{
    let mut f = File::open(path).expect("Error opening classpath file");
    let mut file_contents = String::new();
    f.read_to_string(&mut file_contents).expect("Error reading classpath file");
    let res = file_contents.lines().flat_map(|line|{
        line.split(":")
    }).map(|s| {
        Box::new(s.to_string())
    }).collect();
    res
}

pub fn index_class_path(classfile_path: &Path) -> HashMap<ClassEntry, Box<Path>> {
    let entries= parse_classpath_file(classfile_path);
    entries.iter().flat_map(|s| process_classpath_entry(s)).collect()
}

fn process_classpath_entry(s: &Box<String>) -> Vec<(ClassEntry, Box<Path>)> {
    let mut path_buf = PathBuf::new();
    path_buf.push(s.as_str());
    WalkDir::new(path_buf.clone())
        .min_depth(0)
        .max_depth(1000)
        .into_iter().filter_entry(|_| {
        true
    }).filter(|entry| {
        match entry {
            Ok(f) => {
                let extension = match f.path().extension() {
                    None => { return false },
                    Some(s) => { s },
                };
                f.file_type().is_file() && extension == "class"
            },
            Err(_) => { false },
        }
    }).map(|r| {
        let class_path_entry = r.expect("Error traversing classpath");
        let relative_to_classpath_entry = pathdiff::diff_paths(class_path_entry.path(), path_buf.as_path().clone()).expect("Error indexing classpath");
        let mut package: Vec<String> = relative_to_classpath_entry.iter().map(|sub_package| {
            sub_package.to_str().expect("Strange package name encountered").to_string()
        }).collect();
        package.remove(package.len() - 1);
        let os_str_entry_name = class_path_entry.file_name();
        let name = match os_str_entry_name.to_str() {
            None => { panic!("Strange filename encountered in classpath") },
            Some(s) => { s.replace(".class", "") },
        };
        (ClassEntry { packages: package, name }, class_path_entry.into_path().into_boxed_path())
    }).collect()
}

/*
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
}*/
