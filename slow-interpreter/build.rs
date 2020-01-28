extern crate bindgen;

use std::path::PathBuf;
use std::fs::create_dir;

fn main() {
    let jni_header = env!("JNI_H");
    let jni_md_header = env!("JNI_MD_H");
    println!("cargo:rerun-if-changed={}/{}", jni_header, "jni.h");
    let dl_bindings = bindgen::Builder::default()
        .header("dl-wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .derive_debug(true)
        .rustfmt_bindings(true)
        .generate()
        .expect("Unable to generate bindings");

    let jni_bindings = bindgen::Builder::default()
        .header(format!("{}{}", jni_header, "/jni.h"))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .clang_arg(format!("-I/{}", jni_header))
        .clang_arg(format!("-I/{}", jni_md_header))
        .derive_debug(true)
        .rustfmt_bindings(true)
        .generate()
        .expect("Unable to generate bindings");

    let std_arg = bindgen::Builder::default()
        .header("stdarg-wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .derive_debug(true)
        .rustfmt_bindings(true)
        .generate().unwrap();

    let out_path = PathBuf::from("gen/");
    if !out_path.clone().into_boxed_path().exists() {
        create_dir(out_path.clone().into_boxed_path()).unwrap();
    }

    dl_bindings
        .write_to_file(PathBuf::from("gen/dlopen.rs"))
        .expect("Couldn't write bindings!");

    jni_bindings
        .write_to_file(PathBuf::from("gen/jni.rs"))
        .expect("Couldn't write bindings!");

    std_arg
        .write_to_file(PathBuf::from("gen/stdarg.rs"))
        .expect("Couldn't write bindings!");
}
