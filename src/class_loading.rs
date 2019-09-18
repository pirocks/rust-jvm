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
use std::fmt;

use classfile::{Classfile, parse_class_file};
use classfile::constant_infos::ConstantKind;
use classfile::parsing_util::ParsingContext;
use verification::prolog_info_defs::{class_name, get_super_class_name, extract_string_from_utf8};
use log::{trace, info, warn};
use verification::verify;

#[derive(Eq, PartialEq)]
#[derive(Debug)]
#[derive(Hash)]
pub struct ClassEntry{
    pub name : String,
    pub packages : Vec<String>
}

impl Clone for ClassEntry{
    fn clone(&self) -> Self {
        Self { name: self.name.clone(), packages: self.packages.iter().map(|s|{s.clone()}).collect() }
    }
}

impl std::fmt::Display for ClassEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(")?;
        for s in self.packages.iter() {
            write!(f, "{}.", s)?;
        }
        write!(f, ", {})", self.name)?;
        Ok(())
    }
}

#[derive(Debug)]
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
//    dbg!(extract_string_from_utf8(&classfile.constant_pool[classfile.this_class as usize]));
    let mut name = class_name(classfile);
    class_entry_from_string(&name, false)
}

pub fn class_entry_from_string(str: &String, look_for_java_base: bool) -> ClassEntry{
    let splitted : Vec<String> = str.clone().split('/').map(|s| {s.to_string()}).collect();
    let mut packages = Vec::from(&splitted[0..splitted.len() - 1]);
    if look_for_java_base {
        if let Some(start_of_packages) = packages.iter().position(|s|{**s == "java.base".to_string()}) {
            packages = Vec::from(&packages[(start_of_packages + 1)..packages.len()]);
        } else {
            packages = Vec::new();
        }
    }

    let name = splitted.last().expect("This is a bug").replace(".class", "");//todo validate that this is replacing the last few strings
    ClassEntry {
        packages,name: name.clone()
    }
}

pub fn load_class(classes: &mut JVMClassesState, class_name_with_package : ClassEntry){
    trace!("Starting loading for {}", &class_name_with_package);
    //todo this function is going to be long af
    if classes.using_bootstrap_loader {
        if classes.bootstrap_loaded_classes.contains_key(&class_name_with_package) {
            //so technically here we would need to throw a linkage error or similar
            //however it is convenient to implement like this, so linkage error should be handled by a
            //user facing wrapper.
            return;//class already loaded
        }

//        if classes.loading_in_progress.contains(&class_name_with_package) {
//            unimplemented!("Throw class circularity error.")//todo
//        }

//        dbg!(&classes.indexed_classpath);
//        dbg!(&class_name_with_package);
        let candidate_file = File::open(classes.indexed_classpath.get(&class_name_with_package).unwrap()).expect("Error opening class file");
        let mut p = ParsingContext {f: candidate_file };
        let parsed = parse_class_file(p.borrow_mut());
        if class_name_with_package != class_entry(&parsed){
            dbg!(class_name_with_package);
            dbg!(class_entry(&parsed));
            unimplemented!("Throw no class def found.")
        }
        if classes.bootstrap_loaded_classes.contains_key(&class_name_with_package) {
            dbg!(&classes.bootstrap_loaded_classes);
            dbg!(&class_name_with_package);
            unimplemented!("Throw LinkageError,but this will never happen see above comment")
        }

        //todo use lifetimes instead of clone
        classes.loading_in_progress.insert(class_name_with_package.clone());
        if parsed.super_class == 0 {
            trace!("Parsed Object.class");
        }else{
            let super_class_name = get_super_class_name(&parsed);

            load_class(classes,class_entry_from_string(&super_class_name,false));
            for interface_idx in &parsed.interfaces {
                let interface = match &parsed.constant_pool[*interface_idx as usize].kind {
                    ConstantKind::Class(c) => {c}
                    _ => {panic!()}
                };
                let interface_name = extract_string_from_utf8(&parsed.constant_pool[interface.name_index  as usize]);
                load_class(classes,class_entry_from_string(&interface_name,false))
            };
        }
        match verify(classes){
            None => {
                //class verified successfully.
            },
            Some(s) => {
                load_class(classes,class_entry_from_string(&s,false));
                load_class(classes,class_name_with_package);//todo, fix the concept of class name with package.
            },
        }
        load_verified_class( classes,parsed);
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

