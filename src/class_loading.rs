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


use std::iter::Map;
use std::path::Path;
use std::fs::File;
use std::collections::{HashMap, HashSet};
use classfile::parsing_util::ParsingContext;
use classfile::parse_class_file;
use std::borrow::BorrowMut;
use verification::{class_name, get_super_class_name, extract_string_from_utf8};
use std::hash::Hasher;
use classfile::constant_infos::ConstantKind;

const USING_BOOTSTRAP_LOADER: bool = true;



fn load_class(bootstrap_loaded_classes: &mut HashSet<&str>, indexed_classpath: HashMap<&str,&Path>, class_name_from_file : &str){
    if USING_BOOTSTRAP_LOADER {
        let candidate_file = File::open(indexed_classpath[class_name_from_file]).expect(unimplemented!());
        let mut p = ParsingContext {f: candidate_file };
        let parsed = parse_class_file(p.borrow_mut());
        assert!(class_name_from_file == class_name(&parsed));
        assert!(bootstrap_loaded_classes.contains(class_name_from_file));
//        unimplemented!("Need to check not already loaded and names match class file names");
        if parsed.super_class == 0 {
            unimplemented!("Load Object")
        }else{
            let super_class_name = get_super_class_name(&parsed);
            load_class(bootstrap_loaded_classes,indexed_classpath,super_class_name.as_str());
            unimplemented!("detect cycles");
            for interface_idx in parsed.interfaces {
                let interface = match parsed.constant_pool[interface_idx as usize] {
                    ConstantKind::Class(c) => {c}
                    _ => {panic!()}
                };
                let interface_name = extract_string_from_utf8(&parsed.constant_pool[interface.name_index  as usize]);
                load_class(bootstrap_loaded_classes,indexed_classpath,interface_name.as_str())
            };
        }
        bootstrap_loaded_classes.insert(class_name_from_file)//todo will this include package
    }else {
        unimplemented!()
    }
    return;
}