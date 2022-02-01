fn main() {
    println!("cargo:rustc-cdylib-link-arg=-Wl,-soname,SUNWprivate_1.1");
}
