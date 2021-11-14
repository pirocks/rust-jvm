use std::path::PathBuf;

fn main() {
    let signal = bindgen::Builder::default()
        .header("signals-wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .derive_debug(true)
        .rustfmt_bindings(true)
        .generate()
        .unwrap();

    let ucontext = bindgen::Builder::default()
        .header("ucontext-wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .derive_debug(true)
        .rustfmt_bindings(true)
        .generate()
        .unwrap();

    println!("cargo:rerun-if-changed={}", "signals-wrapper.h");
    println!("cargo:rerun-if-changed={}", "ucontext-wrapper.h");

    signal
        .write_to_file(PathBuf::from("gen/signal.rs"))
        .expect("Couldn't write bindings!");

    ucontext
        .write_to_file(PathBuf::from("gen/ucontext.rs"))
        .expect("Couldn't write bindings!");
}
