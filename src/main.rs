
extern crate argparse;
extern crate log;
extern crate simple_logger;
extern crate rust_jvm_common;

pub mod class_loading;
pub mod classpath_indexing;

use log::{trace, info};

use argparse::{ArgumentParser, Store, StoreTrue};
//use classpath_indexing::index_class_path;


extern crate classfile_parser;
extern crate verification;


fn main() {
    simple_logger::init().unwrap();
    let mut verbose = false;
    let mut debug = false;
    let mut main_class_name = "".to_string();
//    let mut main_class_path = "".to_string();
    let mut class_path_file = "".to_string();

    {  // this block limits scope of borrows by ap.refer() method
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
//        ap.refer(&mut main_class_path)
//            .add_option(&["--main-file"], Store,
//                        "Main class specified as a file path");
//        ap.refer(&mut print_only_mode)
//            .add_option(&["--print-only"], Store,
//                        "only print main class dissasembly.");
        ap.refer(&mut class_path_file)
            .add_option(&["--classpath-file"], Store,
                        "path of file contains class path entries. Separated by :, only include .class files");
//        ap.refer(&mut main_class_name)
//            .add_option(&["--class-path-jar"], Store,
//                        "Include a jar in the classpath");
//        ap.refer(&mut main_class_name)
//            .add_option(&["--class-path-class"], Store,
//                        "Include a class in the classpath");
        ap.parse_args_or_exit();
    }

    trace!("{}",main_class_name);
    trace!("{}",class_path_file);

    if verbose {
        info!("in verbose mode, which currently doesn't do anything, b/c I'm always verbose, since I program in java a lot.");
//        println!("main_class_name is {}", main_class_path);
    }

//    let _indexed_classpath = index_class_path(Path::new(&class_path_file));
    trace!("{}","Indexing complete");
//    let initial_jvm_state = JVMState {
//        using_bootstrap_loader:true,
//        loaders: HashMap::from_iter(vec![(BOOTSTRAP_LOADER_NAME.to_string(), BOOTSTRAP_LOADER.clone())]),
//        indexed_classpath,
//        using_prolog_verifier: false
//    };



//    load_class(&mut initial_jvm_state,class_entry_from_string(&main_class_name,true),true);
//    let main_class_entry = class_entry_from_string(&main_class_name.to_string(), true);
//    unimplemented!("{}",main_class_entry)
//    load_class(&mut initial_jvm_state, BOOTSTRAP_LOADER.clone(), main_class_entry, true);

}

