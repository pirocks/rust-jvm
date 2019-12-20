//steps involved in loading:
//section 5.3.5 in jvm spec
//determining if class is already loaded
//LinkageError on load
//find and load class on classpath
// ClassFormatError on invalid file
//UnsupportedClassVersionError for version shenanigans
//NoClassDefFoundError if other classname does not match filename
//if class has a superclass(or is interface):
use std::sync::Arc;


//load superclass
use std::collections::HashMap;
use std::fs::File;
use log::trace;

use rust_jvm_common::loading::{Loader, ClassEntry, class_entry_from_string, class_entry, JVMState, BOOTSTRAP_LOADER_NAME,BOOTSTRAP_LOADER};
use rust_jvm_common::classfile::{Classfile, ConstantKind};
use rust_jvm_common::utils::extract_string_from_utf8;
use classfile_parser::parse_class_file;
use verification::prolog::prolog_info_writer::get_super_class_name;
use verification::verify;


pub fn load_class(jvm_state: &mut JVMState, loader: Arc<Loader>, to_load: ClassEntry, only_verify: bool) {
    trace!("Starting loading for {}", &to_load);
    if jvm_state.using_bootstrap_loader {
        bootstrap_load(jvm_state, loader, &to_load, only_verify);
    } else {
        unimplemented!()
    }
}

fn bootstrap_load(jvm_state: &mut JVMState, loader: Arc<Loader>, to_load: &ClassEntry, only_verify: bool) {
    bootstrap_load_impl(jvm_state, loader, to_load, only_verify, &mut HashMap::new());
}

fn bootstrap_load_impl(jvm_state: &mut JVMState, loader: Arc<Loader>, to_load: &ClassEntry, only_verify: bool, loading: &mut HashMap<ClassEntry, Arc<Classfile>>) {
    if jvm_state.loaders[&BOOTSTRAP_LOADER_NAME.to_string()].loaded.read().unwrap().contains_key(&to_load) ||
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
    let parsed = parse_class_file(candidate_file,BOOTSTRAP_LOADER.clone());
    if to_load != &class_entry(&parsed) {
        dbg!(to_load);
        dbg!(class_entry(&parsed));
        unimplemented!("Throw no class def found.")
    }
    if jvm_state.loaders[&BOOTSTRAP_LOADER_NAME.to_string()].loaded.read().unwrap().contains_key(&to_load) {
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
    loading.iter().for_each(|(entry,cf)|{
        loader.loading.write().unwrap().insert(entry.clone(),cf.clone());
    });

    match verify(&loading, jvm_state, loader){
        Ok(_) => {unimplemented!()},
        Err(_) => {unimplemented!()},
    }
//    loading.iter().for_each(|(_entry,_cf)|{
//        unimplemented!()
//        loader.loading.write().unwrap().remove(entry.clone(),cf.clone());
//    });
//    if !only_verify {
//        load_verified_class(jvm_state, parsed,);
//    }
//    return ();
}

//fn clinit(class: &Arc<Classfile>, and_then: &fn(&MethodInfo) -> ()) -> () {
//    for method_info in class.methods.iter() {
//        let name = extract_string_from_utf8(&class.constant_pool[method_info.name_index as usize]);
//        if name == "<clinit>" {
//            return and_then(method_info);
//        }
//    };
//    panic!();
//}

//fn load_verified_class(classes: &mut JVMState, loader: &mut Loader, class: Arc<Classfile>) {
//    let entry = class_entry(&class);
//    let after_obtaining_clinit: fn(&MethodInfo) -> () = |m| { unimplemented!()/*run_static_method_no_args(&class, m)*/ };
//    clinit(&class, &after_obtaining_clinit);
//    loader.loaded.write().unwrap().insert(entry, class);
//}



