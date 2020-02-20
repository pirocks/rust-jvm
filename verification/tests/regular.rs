extern crate ntest_timeout;
extern crate timebomb;
extern crate rust_jvm_common;

use ntest_timeout::timeout;
use std::collections::HashMap;
use verification::verifier::TypeSafetyError;
use std::sync::Arc;
use std::path::Path;
use std::path::PathBuf;
use loading::BootstrapLoader;
use std::sync::RwLock;
use loading::Classpath;
use verification::verify;
use verification::VerifierContext;
use rust_jvm_common::classnames::class_name;
use jar_manipulation::JarHandle;
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::loading::LoaderName;
use rust_jvm_common::view::ClassView;

//#[test]
//#[timeout(10000)]
//pub fn can_verify_main() {
//    let main_class_name = "Main".to_string();
//    verify_class_with_name(&main_class_name).unwrap();
//}
//
//#[test]
//#[timeout(10000)]
//pub fn can_verify_float_double_arithmetic() {
//    let main_class_name = "FloatDoubleArithmetic".to_string();
//    verify_class_with_name(&main_class_name).unwrap();
//}
//
//#[test]
//#[timeout(10000)]
//pub fn can_verify_with_main() {
//    let main_class_name = "WithMain".to_string();
//    verify_class_with_name(&main_class_name).unwrap();
//}


#[test]
#[timeout(10000)]
pub fn can_verify_object() {
    let main_class_name = "java/lang/Object".to_string();
    verify_class_with_name(&main_class_name).unwrap();
}


#[test]
#[timeout(30000)]
pub fn can_verify_map() {
    let main_class_name = "java/util/Map".to_string();
    verify_class_with_name(&main_class_name).unwrap();
}

#[test]
#[timeout(30000)]
pub fn can_verify_hashtable_entry_set() {
    let main_class_name = "java/util/Hashtable$EntrySet".to_string();
    verify_class_with_name(&main_class_name).unwrap();
}

#[test]
//#[timeout(30000)]
pub fn can_verify_file_input_stream() {
    let main_class_name = "java/io/FileInputStream".to_string();
    verify_class_with_name(&main_class_name).unwrap();
}


#[test]
#[timeout(30000)]
pub fn can_verify_exceptions() {
    let main_class_name = "java/lang/Throwable".to_string();
    verify_class_with_name(&main_class_name).unwrap();
    let main_class_name = "java/lang/Exception".to_string();
    verify_class_with_name(&main_class_name).unwrap();
    let main_class_name = "java/lang/IllegalArgumentException".to_string();
    verify_class_with_name(&main_class_name).unwrap();
}

#[test]
#[timeout(30000)]
pub fn can_verify_hash_map() {
    let main_class_name = "java/util/HashMap".to_string();
    verify_class_with_name(&main_class_name).unwrap();
}

#[test]
#[timeout(10000)]
pub fn can_verify_system() {
    let main_class_name = "java/lang/System".to_string();
    verify_class_with_name(&main_class_name).unwrap();
}

#[test]
//#[timeout(10000)]
pub fn can_verify_input_stream() {
    let main_class_name = "java/io/InputStream".to_string();
    verify_class_with_name(&main_class_name).unwrap();
}

#[test]
#[timeout(10000)]
pub fn can_verify_print_stream() {
    let main_class_name = "java/io/PrintStream".to_string();
    verify_class_with_name(&main_class_name).unwrap();
}

#[test]
#[timeout(10000)]
pub fn can_verify_security_manger() {
    let main_class_name = "java/lang/SecurityManager".to_string();
    verify_class_with_name(&main_class_name).unwrap();
}

#[test]
#[timeout(10000)]
pub fn can_verify_console() {
    let main_class_name = "java/io/Console".to_string();
    verify_class_with_name(&main_class_name).unwrap();
}


#[test]
#[timeout(20000)]
pub fn can_verify_properties() {
    let main_class_name = "java/util/Properties".to_string();
    verify_class_with_name(&main_class_name).unwrap();
}


#[test]
//#[timeout(10000)]
pub fn can_verify_string() {
    let main_class_name = "java/lang/String".to_string();
    verify_class_with_name(&main_class_name).unwrap();
}

fn verify_class_with_name(main_class_name: &String) -> Result<(), TypeSafetyError> {
    let mut base = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    base.push("../");
    base.push("verification/resources/test");
    base.push("rt.jar");
    dbg!(&base);
    verify_impl(&ClassName::Str(main_class_name.clone()), base.as_path())
}

fn verify_impl(classname: &ClassName, jar_path: &Path) -> Result<(), TypeSafetyError> {
    let loader = BootstrapLoader {
        loaded: RwLock::new(HashMap::new()),
        parsed: RwLock::new(HashMap::new()),
        name: RwLock::new(LoaderName::BootstrapLoader),
        classpath: Classpath { jars: vec![RwLock::new(Box::new(JarHandle::new(jar_path.into()).unwrap()))], classpath_base: vec![] },
    };
    let bootstrap_loader = Arc::new(loader);
    let mut jar_handle = JarHandle::new(jar_path.into()).unwrap();
    let classfile = jar_handle.lookup(classname).unwrap();
    bootstrap_loader.parsed.write().unwrap().insert(class_name(&classfile), classfile.clone());


    match verify(&VerifierContext { bootstrap_loader: bootstrap_loader.clone() }, ClassView::from(classfile.clone()), bootstrap_loader.clone()) {
        Ok(_) => Result::Ok(()),
        Err(err) => {
            match err {
                TypeSafetyError::NotSafe(s) => {
                    dbg!(s);
                    assert!(false);
                    panic!()
                }
                TypeSafetyError::NeedToLoad(ntl) => {
                    dbg!(ntl);
                    assert!(false);
                    panic!();
                }
            }
        }
    }
}