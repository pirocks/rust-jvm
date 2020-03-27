use std::sync::{RwLock, Arc};
use std::path::Path;
use rust_jvm_common::classnames::{ClassName, class_name};
use std::collections::HashMap;
use rust_jvm_common::classfile::Classfile;
use classfile_parser::parse_class_file;
use std::fs::File;
use classfile_view::view::ClassView;
use verification::{verify, VerifierContext};
use log::trace;
use classfile_view::loading::{LoaderName, ClassLoadingError, Loader, LoaderArc, LivePoolGetter};
use jar_manipulation::JarHandle;

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
    fn find_loaded_class(&self, name: &ClassName) -> Option<ClassView> {
        self.loaded.read().unwrap().get(name).cloned().map(|c| ClassView::from(c))
    }

    fn initiating_loader_of(&self, class: &ClassName) -> bool {
        self.loaded.read().unwrap().contains_key(class)
    }

    fn find_representation_of(&self, _class: &ClassName) -> Result<File, ClassLoadingError> {
        unimplemented!()
    }

    fn load_class(&self, self_arc: LoaderArc, class: &ClassName, bl: LoaderArc, live_pool_getter: Arc<dyn LivePoolGetter>) -> Result<ClassView, ClassLoadingError> {
        if !self.initiating_loader_of(class) {
            trace!("loading {}", class.get_referred_name());
            let classfile = self.pre_load(class)?;
            if class != &ClassName::object() {
                if classfile.super_name() == None {
                    self.load_class(self_arc.clone(), &ClassName::object(), bl.clone(),live_pool_getter.clone())?;
                } else {
                    let super_class_name = classfile.super_name();
                    self.load_class(self_arc.clone(), &super_class_name.unwrap(), bl.clone(),live_pool_getter.clone())?;
                }
            }
            for i in classfile.interfaces() {
                let interface_name = i.interface_name();
                self.load_class(self_arc.clone(), &ClassName::Str(interface_name), bl.clone(),live_pool_getter.clone())?;
            }
            match verify(&VerifierContext { live_pool_getter, bootstrap_loader: bl.clone() }, classfile.clone(), self_arc) {
                Ok(_) => {}
                Err(_) => panic!(),
            };
            self.loaded.write().unwrap().insert(class.clone(), classfile.backing_class());
        }
        Result::Ok(ClassView::from(self.loaded.read().unwrap().get(class).unwrap().clone()))
    }

    fn name(&self) -> LoaderName {
        LoaderName::BootstrapLoader
    }

    //todo hacky and janky
    // as a fix for self_arc we could wrap Arc, and have that struct impl loader
    fn pre_load(&self, name: &ClassName) -> Result<ClassView, ClassLoadingError> {
        //todo assert self arc is same
        //todo race potential every time we check for contains_key if there is potential for removal from struct which there may or may not be
        let maybe_classfile: Option<Arc<Classfile>> = self.parsed.read().unwrap().get(name).map(|x| x.clone());
        let res = match maybe_classfile {
            None => {
                let jar_class_file: Option<Arc<Classfile>> = self.classpath.jars.iter().find_map(|h| {
                    let mut h2 = h.write().unwrap();
                    match h2.lookup(name) {
                        Ok(c) => Some(c),
                        Err(_) => None,
                    }
                });
                match jar_class_file {
                    None => {
                        self.search_class_files(name)
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
                Result::Ok(ClassView::from(c))
            }
            Err(e) => {
                dbg!(e);

                dbg!(name);
                Result::Err(ClassLoadingError::ClassNotFoundException)
            }
        }
    }

    fn add_pre_loaded(&self, name: &ClassName, classfile: &Arc<Classfile>) {
        self.parsed.write().unwrap().insert(name.clone(), classfile.clone());
    }
}

impl BootstrapLoader {
    fn search_class_files(&self, name: &ClassName) -> Result<Arc<Classfile>, ClassLoadingError> {
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
                let classfile = parse_class_file(&mut (&file).try_clone().unwrap());
                Result::Ok(Arc::new(classfile))
            }
        }
    }
}

