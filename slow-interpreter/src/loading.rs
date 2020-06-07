use std::sync::{RwLock, Arc};
use std::path::Path;
use rust_jvm_common::classnames::{ClassName, class_name};
use std::collections::HashMap;
use rust_jvm_common::classfile::Classfile;
use classfile_parser::parse_class_file;
use std::fs::File;
use classfile_view::view::ClassView;
use verification::{verify, VerifierContext};
use classfile_view::loading::{LoaderName, ClassLoadingError, Loader, LoaderArc, LivePoolGetter};
use jar_manipulation::JarHandle;
use classfile_view::loading::ClassLoadingError::ClassNotFoundException;

#[derive(Debug)]
pub struct Classpath {
    //base directories to search for a file in.
    pub classpath_base: Vec<Box<Path>>,
    jar_cache: RwLock<HashMap<Box<Path>, Box<JarHandle>>>,
}

impl Classpath {
    pub fn lookup(&self, class_name: &ClassName) -> Result<Arc<Classfile>, ClassLoadingError> {
        for x in &self.classpath_base {
            for dir_member in x.read_dir().unwrap() {
                let dir_member = dir_member.unwrap();
                let is_jar = dir_member.path().extension().map(|x|{&x.to_string_lossy() == "jar"}).unwrap_or(false);
                if is_jar {
                    let mut cache_write_guard = self.jar_cache.write().unwrap();
                    let boxed_path = dir_member.path().into_boxed_path();
                    if cache_write_guard.get(&boxed_path).is_none() {
                        cache_write_guard.insert(boxed_path.clone(), box JarHandle::new(boxed_path).unwrap());
                    }
                }
            }
        }
        let mut cache_read_guard = self.jar_cache.write().unwrap();
        for jar in cache_read_guard.values_mut() {
            match jar.lookup(class_name){
                Ok(c) =>{return Result::Ok(c);},
                Err(_) => {},
            }
        };
        for path in &self.classpath_base {
            let mut new_path = path.clone().into_path_buf();
            new_path.push(format!("{}.class",class_name.get_referred_name()));
            if new_path.is_file(){
                let file_read = &mut File::open(new_path).unwrap();
                let classfile = parse_class_file(file_read);
                return Result::Ok(Arc::new(classfile))
            }
        }
        Result::Err(ClassNotFoundException)
    }

    pub fn from_dirs(dirs: Vec<Box<Path>>) -> Self {
        Self { classpath_base: dirs, jar_cache: RwLock::new(HashMap::new()) }
    }

    pub fn classpath_string(&self) -> String {
        let mut res = String::new();
        for p in &self.classpath_base {
            res.push_str(format!("{}:", p.to_str().unwrap()).as_str());
        }
        res
    }
}

#[derive(Debug)]
pub struct BootstrapLoader {
    pub loaded: RwLock<HashMap<ClassName, Arc<Classfile>>>,
    pub parsed: RwLock<HashMap<ClassName, Arc<Classfile>>>,
    pub name: RwLock<LoaderName>,
    //for now the classpath is immutable so no locks are needed.
    pub classpath: Arc<Classpath>,
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
            // trace!("loading {}", class.get_referred_name());
            let classfile = self.pre_load(class)?;
            if class != &ClassName::object() {
                if classfile.super_name() == None {
                    self.load_class(self_arc.clone(), &ClassName::object(), bl.clone(), live_pool_getter.clone())?;
                } else {
                    let super_class_name = classfile.super_name();
                    self.load_class(self_arc.clone(), &super_class_name.unwrap(), bl.clone(), live_pool_getter.clone())?;
                }
            }
            for i in classfile.interfaces() {
                let interface_name = i.interface_name();
                self.load_class(self_arc.clone(), &ClassName::Str(interface_name), bl.clone(), live_pool_getter.clone())?;
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
                self.classpath.lookup(name)
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
    // fn search_class_files(&self, name: &ClassName) -> Result<Arc<Classfile>, ClassLoadingError> {
    //     let found_class_file = self.classpath.classpath_base.iter().map(|x| {
    //         let mut path_buf = x.to_path_buf();
    //         path_buf.push(format!("{}.class", name.get_referred_name()));
    //         path_buf
    //     }).find(|p| {
    //         p.exists()
    //     });
    //     match found_class_file {
    //         None => {
    //             Result::Err(ClassLoadingError::ClassNotFoundException)
    //         }
    //         Some(path) => {
    //             let file = File::open(path).unwrap();
    //             let classfile = parse_class_file(&mut (&file).try_clone().unwrap());
    //             Result::Ok(Arc::new(classfile))
    //         }
    //     }
    // }
}

