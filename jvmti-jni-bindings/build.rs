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

//TODO TONS OF DUPLICATION WITH JNI-BINDINGS
fn main() {
    let jvm_include_path = env!("JVM_H");
    let jvm_md_include_path = env!("JVM_MD_H");
    let jni_md_include_path = env!("JNI_MD_H");
    let jni_include_path = env!("JNI_H");
    // println!("cargo:rerun-if-changed={}", path_join(jvm_include_path, "/jvm.h"));
    // println!("cargo:rerun-if-changed={}", path_join(jvm_md_include_path, "/jvm_md.h"));
    // println!("cargo:rerun-if-changed={}", path_join(jni_include_path, "/jni.h"));
    // println!("cargo:rerun-if-changed={}", path_join(jni_md_include_path, "/jni_md.h"));
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("{}", jvm_include_path);
    println!("{}", jvm_md_include_path);
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I/{}/", jvm_include_path))
        .clang_arg(format!("-I/{}/", jvm_md_include_path))
        .clang_arg(format!("-I/{}/", jni_include_path))
        .clang_arg(format!("-I/{}/", jni_md_include_path))
        .clang_arg(format!("-I."))
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