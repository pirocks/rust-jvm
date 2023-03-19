extern crate bindgen;

use std::fs::create_dir;
use std::path::PathBuf;

fn main() {
    let dl_bindings = bindgen::Builder::default().header("dl-wrapper.h").parse_callbacks(Box::new(bindgen::CargoCallbacks)).derive_debug(true).rustfmt_bindings(true).generate().expect("Unable to generate bindings");

    let out_path = PathBuf::from("gen/");
    if !out_path.clone().into_boxed_path().exists() {
        create_dir(out_path.into_boxed_path()).unwrap();
    }

    dl_bindings.write_to_file(PathBuf::from("gen/dlopen.rs")).expect("Couldn't write bindings!");
}
