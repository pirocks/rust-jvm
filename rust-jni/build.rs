extern crate bindgen;

//use std::env;
//use std::path::PathBuf;

fn main() {

//    // Tell cargo to invalidate the built crate whenever the wrapper changes
//    let jni_header = env!("JNI_H");
//    println!("cargo:rerun-if-changed={}",jni_header);
////    // The bindgen::Builder is the main entry point
////    // to bindgen, and lets you build up options for
////    // the resulting bindings.
//    let bindings = bindgen::Builder::default()
////        // The input header we would like to generate
////        // bindings for.
//        .header(jni_header)
//        .clang_arg("-I/homes/fpn17/Desktop/jdk8u232-b09/include/linux")
////        // Tell cargo to invalidate the built crate whenever any of the
////        // included header files changed.
//        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
////        // Finish the builder and generate the bindings.
//        .generate()
////        // Unwrap the Result and panic on failure.
//        .expect("Unable to generate bindings");
////
////    // Write the bindings to the $OUT_DIR/bindings.rs file.
//    let out_path = PathBuf::from("src/");
//    bindings
//        .write_to_file(out_path.join("bindings.rs"))
//        .expect("Couldn't write bindings!");
}
