//steps involved in loading:
//section 5.3.5 in jvm spec
//determining if class is already loaded
    //LinkageError on load
//find and load class on classpath
    // ClassFormatError on invalid file
    //UnsupportedClassVersionError for version shenanigans
    //NoClassDefFoundError if other classname does not match filename
    //if class has a superclass(or is interface):
        //load superclass


use std::borrow::BorrowMut;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::Path;

use classfile::{Classfile, parse_class_file};
use classfile::constant_infos::ConstantKind;
use classfile::parsing_util::ParsingContext;
use verification::{class_name, extract_string_from_utf8, get_super_class_name, verify};

struct JVMClassesState {
    //whether we are using bootstrap loader.
    //todo in future there will be map to loader state
    pub using_bootstrap_loader: bool,
    //mapping from full classname(including package) to loaded Classfile
    pub bootstrap_loaded_classes: HashMap<Box<String>,Box<Classfile>>,
    //classes which are being loaded.
    pub loading_in_progress : HashSet<Box<String>>
}

fn load_class(classes: &mut JVMClassesState, indexed_classpath: &HashMap<&str,&Path>, class_name_with_package : &str){
    //todo this function is going to be long af
    if classes.using_bootstrap_loader {
        if classes.bootstrap_loaded_classes.contains_key(&class_name_with_package.to_string()) {
            //so technically here we would need to throw a linkage error or similar
            //however it is convenient to implement like this, so linkage error should be handled by a
            //user facing wrapper.
            return;//class already loaded
        }


        if classes.loading_in_progress.contains(&class_name_with_package.to_string()) {
            unimplemented!("Throw class circularity error.")
        }
        classes.loading_in_progress.insert(Box::new(class_name_with_package.to_string()));

        let candidate_file = File::open(indexed_classpath[class_name_with_package]).expect("Error opening class file");
        let mut p = ParsingContext {f: candidate_file };
        let parsed = parse_class_file(p.borrow_mut());

        if class_name_with_package != class_name(&parsed){
            unimplemented!("Throw no class def found.")
        }
        if !classes.bootstrap_loaded_classes.contains_key(&class_name_with_package.to_string()) {
            unimplemented!("Throw LinkageError")//but this will never happen see above comment
        }

        if parsed.super_class == 0 {
            unimplemented!("Load Object")
        }else{
            let super_class_name = get_super_class_name(&parsed);
            load_class(classes,indexed_classpath,super_class_name.as_str());
            for interface_idx in parsed.interfaces.iter() {
                let interface = match &parsed.constant_pool[*interface_idx as usize].kind {
                    ConstantKind::Class(c) => {c}
                    _ => {panic!()}
                };
                let interface_name = extract_string_from_utf8(&parsed.constant_pool[interface.name_index  as usize]);
                load_class(classes,indexed_classpath,interface_name.as_str())
            };
        }
        verify(&parsed);
        classes.loading_in_progress.remove(&class_name_with_package.to_string());

        classes.bootstrap_loaded_classes.insert(Box::new(class_name_with_package.to_string()), Box::new(parsed));
        //todo add to registry of loaded classes
        return ()
    }else {
        unimplemented!()
    }
    return;
}