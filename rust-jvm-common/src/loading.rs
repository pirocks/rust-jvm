use core::fmt;
use std::sync::{RwLock, Arc};
use std::collections::HashMap;
use crate::classfile::Classfile;
use std::path::{MAIN_SEPARATOR, Path};
use crate::classnames::class_name_legacy;
use crate::classnames::ClassName;
use std::fs::File;
use std::fmt::Display;
use std::fmt::Debug;

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

pub enum ClassLoadingError {
    ClassNotFoundException,

}

#[derive(Debug)]
pub struct Classpath{
    pub name_to_path: HashMap<ClassName,Box<Path>>
}

#[derive(Debug)]
pub struct BootstrapLoader {
    pub loaded: RwLock<HashMap<ClassName, Arc<Classfile>>>,
    pub parsed: RwLock<HashMap<ClassName, Arc<Classfile>>>,
    pub name: RwLock<LoaderName>,
    //for now the classpath is immutable so no locks are needed.
    pub classpath: Classpath
}

#[derive(Debug)]
pub enum LoaderName {
    Str(String),
    BootstrapLoader,
}

impl PartialEq for LoaderName {
    fn eq(&self, other: &LoaderName) -> bool {
        match self {
            LoaderName::Str(s1) => match other {
                LoaderName::Str(s2) => s1 == s2,
                LoaderName::BootstrapLoader => false
            },
            LoaderName::BootstrapLoader => match other {
                LoaderName::Str(_) => false,
                LoaderName::BootstrapLoader => true
            },
        }
    }
}

impl Display for LoaderName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LoaderName::Str(_) => unimplemented!(),
            LoaderName::BootstrapLoader => {
                write!(f, "<bl>")
            }
        }
    }
}

pub trait Loader {
    fn initiating_loader_of(&self, class: &ClassName) -> bool;
    //todo File will have to be a much more general array of bytes
    fn find_representation_of(&self, class: &ClassName) -> Result<File, ClassLoadingError>;
    fn load_class(&self, class: &ClassName) -> Result<Arc<Classfile>, ClassLoadingError>;
    fn name(&self) -> LoaderName;


    //pre loading parses the class file but does not
    fn pre_load(self, name: &ClassName) -> Arc<Classfile>;
}

//todo Loading Constraints

impl Loader for BootstrapLoader {
    fn initiating_loader_of(&self, class: &ClassName) -> bool {
        self.loaded.read().unwrap().contains_key(class)
    }

    fn find_representation_of(&self, class: &ClassName) -> Result<File, ClassLoadingError> {
        unimplemented!()
    }

    fn load_class(&self, class: &ClassName) -> Result<Arc<Classfile>, ClassLoadingError> {
        unimplemented!()
    }



    fn name(&self) -> LoaderName {
        LoaderName::BootstrapLoader
    }

    fn pre_load(self, name: &ClassName) -> Arc<Classfile> {
        //todo race potential every time we check for contains_key if there is potential for removal from struct which there may or may not be
        match self.parsed.read().unwrap().get(name) {
            None => {
                match self.classpath.name_to_path.get(name){
                    None => unimplemented!("{}", "essentially need to handle not knowning of the existence of class referenced by another".to_string()),
                    Some(_path) => {
//                        let p = ParsingContext{
//
//                        };
                        //todo this needs to be somewhere else, to avoid circular deps
                        unimplemented!()
                    },
                }
                unimplemented!()
            }
            Some(c) => c.clone(),
        }
    }
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


//#[derive(Debug)]
pub struct JVMState {
    pub using_bootstrap_loader: bool,
    pub loaders: HashMap<String, Arc<dyn Loader + Send + Sync>>,
    pub indexed_classpath: HashMap<ClassEntry, Box<Path>>,
    pub using_prolog_verifier: bool,
}

pub const BOOTSTRAP_LOADER_NAME: &str = "bl";


lazy_static! {
    pub static ref BOOTSTRAP_LOADER: Arc<dyn Loader + Send + Sync> = Arc::new(BootstrapLoader {
            loaded: RwLock::new(HashMap::new()),
            parsed: RwLock::new(HashMap::new()),
            name: RwLock::new(LoaderName::BootstrapLoader),
            classpath: unimplemented!()
        });

}
