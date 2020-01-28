extern crate log;
extern crate simple_logger;

use std::sync::Arc;
use std::fs::File;
use rust_jvm_common::loading::Loader;
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::loading::ClassLoadingError;
use rust_jvm_common::classfile::Classfile;
use rust_jvm_common::loading::LoaderName;
use std::collections::HashMap;
use std::sync::RwLock;
use std::path::Path;
use rust_jvm_common::classnames::class_name;
use classfile_parser::parse_class_file;
use jar_manipulation::JarHandle;
use verification::verify;
use verification::VerifierContext;
use log::trace;

#[derive(Debug)]
pub struct Classpath {
    pub jars: Vec<RwLock<Box<JarHandle>>>,
    //base directories to search for a file in.
    pub classpath_base: Vec<Box<Path>>,
}

#[derive(Debug)]
pub struct BootstrapLoader {
    pub loaded: RwLock<HashMap<ClassName, Arc<Classfile>>>,
    pub parsed: RwLock<HashMap<ClassName, Arc<Classfile>>>,
    pub name: RwLock<LoaderName>,
    //for now the classpath is immutable so no locks are needed.
    pub classpath: Classpath,
}


impl Loader for BootstrapLoader {
    fn initiating_loader_of(&self, class: &ClassName) -> bool {
        /*{
            self.loaded.read().unwrap().iter().for_each(|x| {
//                dbg!(x.0);
            });

        }*/
        self.loaded.read().unwrap().contains_key(class)
    }

    fn find_representation_of(&self, _class: &ClassName) -> Result<File, ClassLoadingError> {
        unimplemented!()
    }

    fn load_class(&self, self_arc: Arc<dyn Loader + Sync + Send>, class: &ClassName, bl: Arc<dyn Loader + Send + Sync>) -> Result<Arc<Classfile>, ClassLoadingError> {
//        if class == ClassName::object() {
//            panic!()
//        }
        if !self.initiating_loader_of(class) {
            trace!("loading {}", class.get_referred_name());
            let classfile = self.pre_load(self_arc.clone(), class)?;
            if class != &ClassName::object() {
                if classfile.super_class == 0 {
                    self.load_class(self_arc.clone(), &ClassName::object(), bl.clone())?;
                } else {
                    let super_class_name = classfile.super_class_name();
                    self.load_class(self_arc.clone(), &super_class_name, bl.clone())?;
                }
            }
            match verify(&VerifierContext { bootstrap_loader: bl.clone() }, classfile.clone(), self_arc) {
                Ok(_) => {}
                Err(_) => panic!(),
            };
            self.loaded.write().unwrap().insert(class.clone(), classfile);
        }
        Result::Ok(self.loaded.read().unwrap().get(class).unwrap().clone())
    }

    fn name(&self) -> LoaderName {
        LoaderName::BootstrapLoader
    }

    //todo hacky and janky
    // as a fix for self_arc we could wrap Arc, and have that struct impl loader
    fn pre_load(&self, self_arc: Arc<dyn Loader + Sync + Send>, name: &ClassName) -> Result<Arc<Classfile>, ClassLoadingError> {
        //todo assert self arc is same
        //todo race potential every time we check for contains_key if there is potential for removal from struct which there may or may not be
        let maybe_classfile: Option<Arc<Classfile>> = self.parsed.read().unwrap().get(name).map(|x| x.clone());
        let res = match maybe_classfile {
            None => {
                let jar_class_file: Option<Arc<Classfile>> = self.classpath.jars.iter().find_map(|h| {
                    let mut h2 = h.write().unwrap();
                    match h2.lookup(name, self_arc.clone()) {
                        Ok(c) => Some(c),
                        Err(_) => None,
                    }
                });
                match jar_class_file {
                    None => {
                        self.search_class_files(self_arc, name)
                    }
                    Some(c) => {
                        Result::Ok(c)
                    }
                }
            }
            Some(c) => Result::Ok(c.clone()),
        };
        match res {
            Ok(c) => {
                self.parsed.write().unwrap().insert(class_name(&c), c.clone());
                Result::Ok(c)
            }
            Err(_) => res,
        }
    }
}

impl BootstrapLoader {
    fn search_class_files(&self, self_arc: Arc<dyn Loader + Send + Sync>, name: &ClassName) -> Result<Arc<Classfile>, ClassLoadingError> {
        let found_class_file = self.classpath.classpath_base.iter().map(|x| {
            let mut path_buf = x.to_path_buf();
            path_buf.push(format!("{}.class", name.get_referred_name()));
            path_buf
        }).find(|p| {
            p.exists()
        });
        match found_class_file {
            None => {
//                dbg!(name);
                Result::Err(ClassLoadingError::ClassNotFoundException)
            }
            Some(path) => {
                let file = File::open(path).unwrap();
                let classfile = parse_class_file(&mut (&file).try_clone().unwrap(), self_arc);
                Result::Ok(classfile)
            }
        }
    }
}
