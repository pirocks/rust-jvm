extern crate bindgen;

use std::path::PathBuf;
use std::fs::create_dir;

fn main() {
    let dl_bindings = bindgen::Builder::default()
        .header("dl-wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
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

    let signal = bindgen::Builder::default()
        .header("signals-wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .derive_debug(true)
        .rustfmt_bindings(true)
        .generate().unwrap();

    println!("cargo:rerun-if-changed={}","signals-wrapper.h");

    let out_path = PathBuf::from("gen/");
    if !out_path.clone().into_boxed_path().exists() {
        create_dir(out_path.clone().into_boxed_path()).unwrap();
    }

    dl_bindings
        .write_to_file(PathBuf::from("gen/dlopen.rs"))
        .expect("Couldn't write bindings!");

    std_arg
        .write_to_file(PathBuf::from("gen/stdarg.rs"))
        .expect("Couldn't write bindings!");

    signal
        .write_to_file(PathBuf::from("gen/signal.rs"))
        .expect("Couldn't write bindings!");
}
