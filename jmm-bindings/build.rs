extern crate bindgen;

use std::env;
use std::fs::create_dir;
use std::path::PathBuf;

// fn path_join(one: &str, two: &str) -> String {
//     let mut path = PathBuf::new();
//     path.push(one);
//     path.push(two);
//     let res = path.as_path().to_str().unwrap();
//     res.to_string()
// }

fn main() {
    let jmm_include_path = env::var("JMM_H").unwrap_or("/home/francis/build/openjdk-debug/jdk8u/jdk/src/share/javavm/export/".to_string());
    // println!("cargo:rerun-if-changed={}", path_join(jvm_include_path, "/jvm.h"));
    // println!("cargo:rerun-if-changed={}", path_join(jvm_md_include_path, "/jvm_md.h"));
    // println!("cargo:rerun-if-changed={}", path_join(jni_include_path, "/jni.h"));
    // println!("cargo:rerun-if-changed={}", path_join(jni_md_include_path, "/jni_md.h"));
    println!("cargo:rerun-if-changed=wrapper.h");
    // println!("{}", jvm_include_path);
    // println!("{}", jvm_md_include_path);
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I/{}/", jmm_include_path))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .derive_debug(true)
        .rustfmt_bindings(true)
        .generate()
        .expect("Unable to generate bindings");
    let out_path = PathBuf::from("gen/");
    if !out_path.clone().into_boxed_path().exists() {
        create_dir(out_path.clone().into_boxed_path()).unwrap();
    }
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}