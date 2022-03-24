use libloading::Library;
use jvmti_jni_bindings::{jint};
use slow_interpreter::rust_jni::dlopen::{RTLD_GLOBAL, RTLD_LAZY};

fn main(){
    let lib = Library::new("libjvm.so", (RTLD_LAZY | RTLD_GLOBAL) as i32).unwrap();
    let real_main = unsafe { lib.get::<fn() -> jint>("rust_jvm_real_main".as_bytes()) }.unwrap();
    real_main();
}