extern crate argparse;
extern crate log;
extern crate simple_logger;
extern crate rust_jvm_common;

pub mod class_loading;
pub mod classpath_indexing;

use log::info;

use argparse::{ArgumentParser, Store, StoreTrue, List};
use std::path::Path;
use slow_interpreter::{run, JVMOptions};
use rust_jvm_common::classnames::ClassName;
use slow_interpreter::loading::Classpath;


extern crate classfile_parser;
extern crate verification;
extern crate slow_interpreter;
extern crate jar_manipulation;
extern crate classfile_view;


fn main() {
    simple_logger::init().unwrap();
    let mut verbose = false;
    let mut debug = false;
    let mut main_class_name = "".to_string();
    let mut class_entries: Vec<String> = vec![];
    let mut args: Vec<String> = vec![];
    let mut libjava: String = "".to_string();
    let mut enable_tracing = false;
    let mut enable_jvmti = false;
    let mut libjdwp: String = "/home/francis/build/openjdk-jdk8u/build/linux-x86_64-normal-server-release/jdk/lib/amd64/libjdwp.so".to_string();
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
        // ap.refer(&mut jars)
        //     .add_option(&["--jars"], List, "A list of jars from which to load classes");
        ap.refer(&mut class_entries)
            .add_option(&["--classpath"], List, "A list of directories from which to load classes");
        ap.refer(&mut args)
            .add_option(&["--args"], List, "A list of args to pass to main");
        ap.refer(&mut libjava).add_option(&["--libjava"], Store, "");
        ap.refer(&mut libjdwp).add_option(&["--libjdwp"], Store, "");
        ap.refer(&mut enable_tracing).add_option(&["--tracing"],StoreTrue,"Enable debug tracing");
        ap.refer(&mut enable_jvmti).add_option(&["--jvmti"],StoreTrue,"Enable JVMTI");
        ap.parse_args_or_exit();
    }

    if verbose {
        info!("in verbose mode, which currently doesn't really do anything, b/c I'm always verbose, since I program in java a lot.");
    }


    let classpath = Classpath::from_dirs(class_entries.iter().map(|x| Path::new(x).into()).collect());
    let main_class_name = ClassName::Str(main_class_name.replace('.', "/"));
    let jvm_options = JVMOptions::new(main_class_name, classpath, args ,libjava, libjdwp,enable_tracing,enable_jvmti);

    run(jvm_options).unwrap();
}

