extern crate ntest_timeout;
extern crate timebomb;
use std::path::Path;
use jar_manipulation::JarHandle;
use rust_jvm_common::classnames::ClassName;
use std::sync::Arc;
use rust_jvm_common::loading::EmptyLoader;

#[test]
//#[timeout(10000)]
pub fn can_open_rt_jar() {
    let p = Path::new("/homes/fpn17/Desktop/jdk8u232-b09/jre/lib/rt.jar");
    let mut z = JarHandle::new(p.into()).unwrap().zip_archive;
    for i in 0..z.len(){
        dbg!(z.by_index(i).unwrap().name());
    }

}

#[test]
pub fn can_get_object() {
    let p = Path::new("/homes/fpn17/Desktop/jdk8u232-b09/jre/lib/rt.jar");
    let mut j = JarHandle::new(p.into()).unwrap();
    j.lookup(ClassName::Str("java/lang/Object".to_string()),Arc::new(EmptyLoader{})).unwrap();

}