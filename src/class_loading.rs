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


use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs::File;
use std::path::{MAIN_SEPARATOR, Path};

use log::trace;

use classfile::{Classfile, MethodInfo, parse_class_file};
use classfile::constant_infos::ConstantKind;
use verification::prolog_info_writer::{class_name, extract_string_from_utf8, get_super_class_name};
use verification::verify;
use std::rc::Rc;
use classfile::parsing_util::ParsingContext;

#[derive(Eq, PartialEq)]
#[derive(Debug)]
#[derive(Hash)]
pub struct ClassEntry {
    pub name: String,
    pub packages: Vec<String>,
}

impl Clone for ClassEntry {
    fn clone(&self) -> Self {
        Self { name: self.name.clone(), packages: self.packages.iter().map(|s| { s.clone() }).collect() }
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
    pub bootstrap_loaded_classes: HashMap<ClassEntry, Rc<Classfile>>,
    //classes which are being loaded.
    pub loading_in_progress: HashSet<ClassEntry>,
    pub partial_load: HashSet<ClassEntry>,
    //where classes are
    pub indexed_classpath: HashMap<ClassEntry, Box<Path>>,
}

fn class_entry(classfile: &Classfile) -> ClassEntry {
    let name = class_name(classfile);
    class_entry_from_string(&name, false)
}

pub fn class_entry_from_string(str: &String, use_dots: bool) -> ClassEntry {
    let split_on = if use_dots { '.' } else { MAIN_SEPARATOR };
    let splitted: Vec<String> = str.clone().split(split_on).map(|s| { s.to_string() }).collect();
    let packages = Vec::from(&splitted[0..splitted.len() - 1]);
    let name = splitted.last().expect("This is a bug").replace(".class", "");//todo validate that this is replacing the last few chars
    ClassEntry {
        packages,
        name: name.clone(),
    }
}

pub fn load_class(classes: &mut JVMClassesState, class_name_with_package: ClassEntry, only_verify: bool) {
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
        let path_of_class_to_load = classes.indexed_classpath.get(&class_name_with_package).or_else(|| {
            trace!("Unable to find: {}", &class_name_with_package);
            dbg!(&class_name_with_package);
            panic!();
        }).unwrap();

        let candidate_file = File::open(path_of_class_to_load).expect("Error opening class file");
//        let mut p = ParsingContext { f: candidate_file ,constant_pool:None};
        let parsed = parse_class_file(&mut ParsingContext { f:candidate_file } );
        if class_name_with_package != class_entry(&parsed) {
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
        } else {
            let super_class_name = get_super_class_name(&parsed);

            load_class(classes, class_entry_from_string(&super_class_name, false), only_verify);
            for interface_idx in &parsed.interfaces {
                let interface = match &parsed.constant_pool[*interface_idx as usize].kind {
                    ConstantKind::Class(c) => { c }
                    _ => { panic!() }
                };
                let interface_name = extract_string_from_utf8(&parsed.constant_pool[interface.name_index as usize]);
                load_class(classes, class_entry_from_string(&interface_name, false), only_verify)
            };
        }
        match verify(classes) {
            None => {
                //class verified successfully.
            }
            Some(s) => {
                classes.partial_load.insert(class_entry_from_string(&s, false));
                load_class(classes, class_name_with_package, only_verify);
            }
        }
        if !only_verify {
            load_verified_class(classes, parsed);
        }
        return ();
    } else {
        unimplemented!()
    }
}

fn clinit(class: &Rc<Classfile>, and_then: &fn(&MethodInfo) -> ()) -> () {
    for method_info in class.methods.borrow().iter() {
        let name = extract_string_from_utf8(&class.constant_pool[method_info.name_index as usize]);
        if name == "<clinit>" {
            return and_then(method_info);
        }
    };
    panic!();
}

fn load_verified_class(classes: &mut JVMClassesState, class: Rc<Classfile>) {
    let entry = class_entry(&class);
    classes.loading_in_progress.remove(&entry);
    let after_obtaining_clinit:fn(&MethodInfo) -> () = |m| { unimplemented!()/*run_static_method_no_args(&class, m)*/ };
    clinit(&class, &after_obtaining_clinit);
    classes.bootstrap_loaded_classes.insert(entry, class);
}

