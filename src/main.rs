use std::path::PathBuf;
use std::process::exit;
use std::time::Duration;
use libloading::Library;
use libloading::os::unix::{RTLD_GLOBAL, RTLD_LAZY};

#[allow(unused)]
fn libjvm_path_from_java_home() -> anyhow::Result<Option<PathBuf>>{
    match std::env::var("JAVA_HOME"){
        Ok(java_home) => {
            todo!()
        }
        Err(_) => Ok(None)
    }
}

fn main() {
    let lib = Library::new("libjvm.so", (RTLD_LAZY | RTLD_GLOBAL) as i32).unwrap();
    let real_main = unsafe { lib.get::<fn()>("rust_jvm_real_main".as_bytes()) }.unwrap();
    real_main();
}