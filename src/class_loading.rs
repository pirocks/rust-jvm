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

#[derive(Eq, PartialEq)]
#[derive(Hash)]
pub struct ClassEntry{
    pub name : String,
    pub packages : Vec<String>
}

pub struct JVMClassesState {
    //whether we are using bootstrap loader.
    //todo in future there will be map to loader state
    pub using_bootstrap_loader: bool,
    //mapping from full classname(including package) to loaded Classfile
    pub bootstrap_loaded_classes: HashMap<ClassEntry,Box<Classfile>>,
    //classes which are being loaded.
    pub loading_in_progress : HashSet<ClassEntry>,
    //where classes are
    pub indexed_classpath: HashMap<ClassEntry,Box<Path>>
}

fn class_entry(classfile: &Classfile) -> ClassEntry{
    unimplemented!()
}

fn class_entry_from_string(str: String) -> ClassEntry{
    let splitted : Vec<String> = str.split('/').map(|s| {s.to_string()}).collect();
    let packages = Vec::from(&splitted[0..splitted.len() - 1]);
    let name = splitted.last().expect("This is a bug");
    ClassEntry {
        packages,name: name.clone()
    }
}

fn load_class(classes: &mut JVMClassesState, class_name_with_package : ClassEntry){
    //todo this function is going to be long af
    if classes.using_bootstrap_loader {
        if classes.bootstrap_loaded_classes.contains_key(&class_name_with_package) {
            //so technically here we would need to throw a linkage error or similar
            //however it is convenient to implement like this, so linkage error should be handled by a
            //user facing wrapper.
            return;//class already loaded
        }


        if classes.loading_in_progress.contains(&class_name_with_package) {
            unimplemented!("Throw class circularity error.")
        }

        let candidate_file = File::open(classes.indexed_classpath.get(&class_name_with_package).unwrap()).expect("Error opening class file");
        let mut p = ParsingContext {f: candidate_file };
        let parsed = parse_class_file(p.borrow_mut());
        if class_name_with_package != class_entry(&parsed){
            unimplemented!("Throw no class def found.")
        }
        if !classes.bootstrap_loaded_classes.contains_key(&class_name_with_package) {
            unimplemented!("Throw LinkageError")//but this will never happen see above comment
        }



        if parsed.super_class == 0 {
            unimplemented!("Load Object")
        }else{
            let super_class_name = get_super_class_name(&parsed);

            classes.loading_in_progress.insert(class_name_with_package);
            load_class(classes,class_entry_from_string(super_class_name));
            for interface_idx in &parsed.interfaces {
                let interface = match &parsed.constant_pool[*interface_idx as usize].kind {
                    ConstantKind::Class(c) => {c}
                    _ => {panic!()}
                };
                let interface_name = extract_string_from_utf8(&parsed.constant_pool[interface.name_index  as usize]);
                load_class(classes,class_entry_from_string(interface_name))
            };
        }
        verify(classes);
        load_verified_class( classes,parsed);
        //todo add to registry of loaded classes
        return ()
    }else {
        unimplemented!()
    }
}

fn load_verified_class(classes: &mut JVMClassesState,class: Classfile) {
    let entry = class_entry(&class);
    classes.loading_in_progress.remove(&entry);
    classes.bootstrap_loaded_classes.insert(entry, Box::new(class));
}