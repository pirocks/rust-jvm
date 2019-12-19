use core::fmt;
use std::sync::{RwLock, Arc};
use std::collections::HashMap;
use crate::classfile::Classfile;
use std::path::{MAIN_SEPARATOR, Path};
use crate::classnames::class_name_legacy;

#[derive(Eq, PartialEq)]
#[derive(Debug)]
#[derive(Hash)]
pub struct ClassEntry {
    // todo deprecated superseeded by ClassName
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
pub struct Loader {
    //todo look at what spec has to say about this in more detail
    pub loaded: RwLock<HashMap<ClassEntry, Arc<Classfile>>>,
    pub loading: RwLock<HashMap<ClassEntry, Arc<Classfile>>>,
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


#[derive(Debug)]
pub struct JVMState {
    pub using_bootstrap_loader: bool,
    pub loaders: HashMap<String, Arc<Loader>>,
    pub indexed_classpath: HashMap<ClassEntry, Box<Path>>,
    pub using_prolog_verifier: bool,
}
pub const BOOTSTRAP_LOADER_NAME: &str = "bl";

lazy_static! {
    pub static ref BOOTSTRAP_LOADER: Arc<Loader> = Arc::new(Loader {
        loaded: RwLock::new(HashMap::new()),
        loading: RwLock::new(HashMap::new()),
        name: BOOTSTRAP_LOADER_NAME.to_string()
    });

}
