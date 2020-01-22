extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-cdylib-link-arg=-Wl,-soname,SUNWprivate_1.1");
    let jvm_include_path = env!("JVM_H");
    let jvm_md_include_path = env!("JVM_MD_H");
    let jni_md_include_path = env!("JNI_MD_H");
    let jni_include_path = env!("JNI_H");
    println!("cargo:rerun-if-changed={}/jvm.h", jvm_include_path);
    println!("cargo:rerun-if-changed={}/jvm_md.h", jvm_md_include_path);
    println!("cargo:rerun-if-changed={}/jni.h", jni_include_path);
    println!("cargo:rerun-if-changed={}/jni_md.h", jni_md_include_path);
    println!("cargo:rerun-if-changed=wrapper.h");
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I/{}", jvm_include_path))
        .clang_arg(format!("-I/{}", jvm_md_include_path))
        .clang_arg(format!("-I/{}", jni_include_path))
        .clang_arg(format!("-I/{}", jni_md_include_path))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");
    let out_path = PathBuf::from("src/");
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}