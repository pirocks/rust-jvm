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


use std::cell::RefCell;
use std::collections::{HashMap};
use std::fmt;
use std::fs::File;
use std::path::{MAIN_SEPARATOR, Path};
use std::rc::Rc;

use log::trace;

use classfile::{Classfile, MethodInfo, parse_class_file};
use classfile::constant_infos::ConstantKind;
use classfile::parsing_util::ParsingContext;
use verification::prolog_info_writer::{class_name_legacy, extract_string_from_utf8, get_super_class_name};
use verification::verifier::TypeSafetyResult;
use verification::verify;

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
pub struct JVMState {
    pub using_bootstrap_loader: bool,
    pub loaders: HashMap<String, Rc<Loader>>,
    pub indexed_classpath: HashMap<ClassEntry, Box<Path>>,
    pub using_prolog_verifier: bool
}


#[derive(Debug)]
pub struct Loader {
    //todo look at what spec has to say about this in more detail
    pub loaded: RefCell<HashMap<ClassEntry, Rc<Classfile>>>,
    pub loading: RefCell<HashMap<ClassEntry, Rc<Classfile>>>,
    pub name: String,
}

pub fn class_entry(classfile: &Classfile) -> ClassEntry {
    let name = class_name_legacy(classfile);
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

const BOOTSTRAP_LOADER_NAME: &str = "bl";

pub fn load_class(jvm_state: &mut JVMState, loader: Rc<Loader>, to_load: ClassEntry, only_verify: bool) {
    trace!("Starting loading for {}", &to_load);
    if jvm_state.using_bootstrap_loader {
        bootstrap_load(jvm_state, loader, &to_load, only_verify);
    } else {
        unimplemented!()
    }
}

fn bootstrap_load(jvm_state: &mut JVMState, loader: Rc<Loader>, to_load: &ClassEntry, only_verify: bool) {
    bootstrap_load_impl(jvm_state, loader, to_load, only_verify, &mut HashMap::new());
}

fn bootstrap_load_impl(jvm_state: &mut JVMState, loader: Rc<Loader>, to_load: &ClassEntry, only_verify: bool, loading: &mut HashMap<ClassEntry, Rc<Classfile>>) {
    if jvm_state.loaders[&BOOTSTRAP_LOADER_NAME.to_string()].loaded.borrow().contains_key(&to_load) ||
        loading.contains_key(to_load) {
        //so technically here we would need to throw a linkage error or similar
        //however it is convenient to implement like this, so linkage error should be handled by a
        //user facing wrapper.
        return;//class already loaded
    }
//        if classes.loading_in_progress.contains(&class_name_with_package) {
//            unimplemented!("Throw class circularity error.")//todo
//        }
    let path_of_class_to_load = jvm_state.indexed_classpath.get(&to_load).or_else(|| {
        trace!("Unable to find: {}", &to_load);
        dbg!(&to_load);
        panic!();
    }).unwrap();
    let candidate_file = File::open(path_of_class_to_load).expect("Error opening class file");
    let parsed = parse_class_file(&mut ParsingContext { f: candidate_file });
    if to_load != &class_entry(&parsed) {
        dbg!(to_load);
        dbg!(class_entry(&parsed));
        unimplemented!("Throw no class def found.")
    }
    if jvm_state.loaders[&BOOTSTRAP_LOADER_NAME.to_string()].loaded.borrow().contains_key(&to_load) {
        dbg!(&jvm_state.loaders[&BOOTSTRAP_LOADER_NAME.to_string()].loaded);
        dbg!(&to_load);
        unimplemented!("Throw LinkageError,but this will never happen")
    }
//todo use lifetimes instead of clone
//        jvm_state.loaders[BOOTSTRAP_LOADER_NAME].loaded.insert(class_name_with_package,);
    loading.insert(to_load.clone(), parsed.clone());
    if parsed.super_class == 0 {
        trace!("Parsed Object.class");
    } else {
        let super_class_name = get_super_class_name(&parsed);
        let super_class_entry = class_entry_from_string(&super_class_name, false);
        bootstrap_load_impl(jvm_state, loader.clone(), &super_class_entry, only_verify, loading);
        for interface_idx in &parsed.interfaces {
            let interface = match &parsed.constant_pool[*interface_idx as usize].kind {
                ConstantKind::Class(c) => { c }
                _ => { panic!() }
            };
            let interface_name = extract_string_from_utf8(&parsed.constant_pool[interface.name_index as usize]);
            let interface_entry = class_entry_from_string(&interface_name, false);
            bootstrap_load_impl(jvm_state, loader.clone(), &interface_entry, only_verify, loading)
        };
    }
    match verify(&loading, jvm_state, loader) {
        TypeSafetyResult::NotSafe(_) => {}
        TypeSafetyResult::Safe() => {}
        TypeSafetyResult::NeedToLoad(_) => {}
    }
//    if !only_verify {
//        load_verified_class(jvm_state, parsed,);
//    }
    return ();
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

fn load_verified_class(classes: &mut JVMState, loader: &mut Loader, class: Rc<Classfile>) {
    let entry = class_entry(&class);
    let after_obtaining_clinit:fn(&MethodInfo) -> () = |m| { unimplemented!()/*run_static_method_no_args(&class, m)*/ };
    clinit(&class, &after_obtaining_clinit);
    let mut old_map = loader.loaded.borrow_mut();
    old_map.insert(entry, class);
    loader.loaded.replace(old_map.clone());//todo get rid
}

