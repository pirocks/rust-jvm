
extern crate argparse;
extern crate classfile;


use argparse::{ArgumentParser, Store, StoreTrue};

fn main() {
    let mut verbose = false;
    let mut debug = false;
    let mut print_only_mode = false;
    let mut main_class_name = "".to_string();
    let mut main_class_path = "".to_string();

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
        ap.refer(&mut main_class_path)
            .add_option(&["--main-file"], Store,
                        "Main class specified as a file path");
        ap.refer(&mut print_only_mode)
            .add_option(&["--print-only"], Store,
                        "only print main class dissasembly.");
//        ap.refer(&mut main_class_name)
//            .add_option(&["--class-path-jar"], Store,
//                        "Include a jar in the classpath");
//        ap.refer(&mut main_class_name)
//            .add_option(&["--class-path-class"], Store,
//                        "Include a class in the classpath");
        ap.parse_args_or_exit();
    }

    if verbose {
        println!("main_class_name is {}", main_class_name);
    }

}