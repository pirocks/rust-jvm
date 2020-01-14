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

#[derive(Debug)]
pub struct Classpath {
    //base directories to search for a file in.
    pub classpath_base: Vec<Box<Path>>
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
        self.loaded.read().unwrap().contains_key(class)
    }

    fn find_representation_of(&self, _class: &ClassName) -> Result<File, ClassLoadingError> {
        unimplemented!()
    }

    fn load_class(&self, _class: &ClassName) -> Result<Arc<Classfile>, ClassLoadingError> {
        unimplemented!()
    }

    fn name(&self) -> LoaderName {
        LoaderName::BootstrapLoader
    }

    //todo hacky and janky
    fn pre_load(&self, self_arc: Arc<dyn Loader + Sync + Send>, name: &ClassName) -> Result<Arc<Classfile>, ClassLoadingError> {
        //todo race potential every time we check for contains_key if there is potential for removal from struct which there may or may not be
        let maybe_classfile: Option<Arc<Classfile>> = self.parsed.read().unwrap().get(name).map(|x| x.clone());
        match maybe_classfile {
            None => {
                let found_class_file = self.classpath.classpath_base.iter().map(|x| {
                    let mut path_buf = x.to_path_buf();
                    path_buf.push(format!("{}.class", name.get_referred_name()));
                    path_buf
                }).find(|p| {
                    p.exists()
                });
                match found_class_file {
                    None => {
                        dbg!(name);
                        Result::Err(ClassLoadingError::ClassNotFoundException)
                    }
                    Some(path) => {
                        let file = File::open(path).unwrap();
                        let classfile = parse_class_file((&file).try_clone().unwrap(), self_arc);
                        self.parsed.write().unwrap().insert(class_name(&classfile), classfile.clone());
                        Result::Ok(classfile)
                    }
                }
            }
            Some(c) => Result::Ok(c.clone()),
        }
    }
}