use std::env;
use std::fs::create_dir;
use std::path::PathBuf;

use xtask::load_xtask_config;

fn this_dir() -> PathBuf {
    PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set?"))
}

fn workspace_dir() -> PathBuf {
    this_dir().parent().unwrap().to_path_buf()
}


// fn path_join(a: impl AsRef<str>, b: impl AsRef<str>) -> String{
//     let mut string = a.as_ref().to_string();
//     string.push_str(b.as_ref());
//     string
// }

fn main() -> anyhow::Result<()> {
    eprintln!("If you see failures here make sure you have run `cargo xtask deps`");
    let workspace_dir: PathBuf = workspace_dir();
    let xtask = load_xtask_config(&workspace_dir)?.expect("No xtask config found.");
    let dep_dir: PathBuf = xtask.dep_dir;
    let jdk_source_dir = dep_dir.join("jdk8u");

    //todo use join here
    let jmm_include_path = env::var("JMM_H").unwrap_or(format!("{}/jdk/src/share/javavm/export/", jdk_source_dir.display()));
    let jvm_include_path = env::var("JVM_H").unwrap_or(format!("{}/jdk/src/share/javavm/export/", jdk_source_dir.display()));
    let jvm_md_include_path = env::var("JVM_MD_H").unwrap_or(format!("{}/jdk/src/solaris/javavm/export/", jdk_source_dir.display()));
    let jni_md_include_path = env::var("JNI_MD_H").unwrap_or(format!("{}/build/linux-x86_64-normal-server-fastdebug/jdk/include/linux/", jdk_source_dir.display()));
    let jni_include_path = env::var("JNI_H").unwrap_or(format!("{}/build/linux-x86_64-normal-server-fastdebug/jdk/include/linux/", jdk_source_dir.display()));
    // println!("cargo:rerun-if-changed={}", path_join(&jvm_include_path, "/jvm.h"));
    // println!("cargo:rerun-if-changed={}", path_join(&jvm_md_include_path, "/jvm_md.h"));
    // println!("cargo:rerun-if-changed={}", path_join(&jni_include_path, "/jni.h"));
    // println!("cargo:rerun-if-changed={}", path_join(&jni_md_include_path, "/jni_md.h"));
    println!("cargo:rerun-if-changed=wrapper.h");
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I/{}/", jmm_include_path))
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
    bindings.write_to_file(out_path.join("bindings.rs")).expect("Couldn't write bindings!");
    Ok(())
}
