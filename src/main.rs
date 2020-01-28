extern crate argparse;
extern crate log;
extern crate simple_logger;
extern crate rust_jvm_common;

pub mod class_loading;
pub mod classpath_indexing;

use log::{trace, info};

use argparse::{ArgumentParser, Store, StoreTrue, List};
use loading::Classpath;
use std::path::Path;
use loading::BootstrapLoader;
use std::sync::RwLock;
use std::collections::HashMap;
use rust_jvm_common::loading::LoaderName;
use slow_interpreter::run;
use rust_jvm_common::classnames::ClassName;
use jar_manipulation::JarHandle;
use std::sync::Arc;
use slow_interpreter::rust_jni::new_java_loading;


extern crate classfile_parser;
extern crate verification;
extern crate loading;
extern crate slow_interpreter;
extern crate jar_manipulation;
//extern crate rust_jni;


fn main() {
    simple_logger::init().unwrap();
    let mut verbose = false;
    let mut debug = false;
    let mut main_class_name = "".to_string();
    let mut jars: Vec<String> = vec![];
    let mut class_entries: Vec<String> = vec![];
    let mut args: Vec<String> = vec![];
    let mut libjava: String = "".to_string();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("A jvm written partially in rust");
        ap.refer(&mut verbose)
            .add_option(&["-v", "--verbose"], StoreTrue,
                        "Be verbose");
        ap.refer(&mut debug).add_option(&["-v", "--verbose"], StoreTrue,
                                        "Log debug info");
        ap.refer(&mut main_class_name)
            .add_option(&["--main"], Store,
                        "Main class");
        ap.refer(&mut jars)
            .add_option(&["--jars"], List, "A list of jars from which to load classes");
        ap.refer(&mut class_entries)
            .add_option(&["--classpath"], List, "A list of directories from which to load classes");
        ap.refer(&mut args)
            .add_option(&["--args"], List, "A list of args to pass to main");
        ap.refer(&mut libjava).add_option(&["--libjava"], Store, "");
        ap.parse_args_or_exit();
    }

    if verbose {
        info!("in verbose mode, which currently doesn't do anything, b/c I'm always verbose, since I program in java a lot.");
    }

    let classpath = Classpath {
        jars: jars.iter().map(|x| {
            let path = Path::new(x).into();
            let jar_handle = JarHandle::new(path).unwrap();
            RwLock::new(Box::new(jar_handle))
        }).collect(),
        classpath_base: class_entries.iter().map(|x| Path::new(x).into()).collect(),
    };

    trace!("Classpath parsed and loaded");

    let bootstrap_loader = BootstrapLoader {
        loaded: RwLock::new(HashMap::new()),
        parsed: RwLock::new(HashMap::new()),
        name: RwLock::new(LoaderName::BootstrapLoader),
        classpath,
    };

    trace!("Bootstrap Loader created");


    let main_class_name = ClassName::Str(main_class_name.replace('.', "/"));
    trace!("Loading main class: {:?}", main_class_name);
    //todo I guess the bootstrap loader doesn't need to be Arc
    let jni = new_java_loading(libjava);
    run(&main_class_name, Arc::new(bootstrap_loader), args, jni).unwrap();
}

