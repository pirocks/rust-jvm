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

#[derive(Debug)]
pub struct Classpath{
    //base directories to search for a file in.
    pub classpath_base : Vec<Box<Path>>
}

#[derive(Debug)]
pub struct BootstrapLoader {
    pub loaded: RwLock<HashMap<ClassName, Arc<Classfile>>>,
    pub parsed: RwLock<HashMap<ClassName, Arc<Classfile>>>,
    pub name: RwLock<LoaderName>,
    //for now the classpath is immutable so no locks are needed.
    pub classpath: Classpath
}


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

    fn pre_load(&self, name: &ClassName) -> Result<Arc<Classfile>,ClassLoadingError> {
        //todo race potential every time we check for contains_key if there is potential for removal from struct which there may or may not be
        match self.parsed.read().unwrap().get(name) {
            None => {
                unimplemented!("{:?}",name);

//                match self.classpath.name_to_path.get(name){
//                    None => unimplemented!("{}", "essentially need to handle not knowning of the existence of class referenced by another".to_string()),
//                    Some(_path) => {
//                        let p = ParsingContext{
//
//                        };
//                        todo this needs to be somewhere else, to avoid circular deps
//                        unimplemented!()
//                    },
//                }
            }
            Some(c) => Result::Ok(c.clone()),
        }
    }
}


//lazy_static! {
//    pub static ref BOOTSTRAP_LOADER: Arc<dyn Loader + Send + Sync> = Arc::new(BootstrapLoader {
//            loaded: RwLock::new(HashMap::new()),
//            parsed: RwLock::new(HashMap::new()),
//            name: RwLock::new(LoaderName::BootstrapLoader),
//            classpath: Classpath {name_to_path:HashMap::new()}
//        });
//}
