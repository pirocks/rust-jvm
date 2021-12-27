use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::sync::{Arc, RwLock};
use iced_x86::OpCodeOperandKind::cl;

use classfile_parser::parse_class_file;
use jar_manipulation::JarHandle;
use rust_jvm_common::classfile::Classfile;
use rust_jvm_common::compressed_classfile::CompressedClassfileStringPool;
use rust_jvm_common::compressed_classfile::names::CClassName;
use rust_jvm_common::loading::ClassLoadingError;
use rust_jvm_common::loading::ClassLoadingError::ClassNotFoundException;

#[derive(Debug)]
pub struct Classpath {
    //base directories to search for a file in.
    pub classpath_base: Vec<Box<Path>>,
    jar_cache: RwLock<HashMap<Box<Path>, Box<JarHandle<File>>>>,
    class_cache: RwLock<HashMap<CClassName, Arc<Classfile>>>, //todo deal with multiple entries with same name
}

impl Classpath {
    pub fn lookup(&self, class_name: &CClassName, pool: &CompressedClassfileStringPool) -> Result<Arc<Classfile>, ClassLoadingError> {
        let mut guard = self.class_cache.write().unwrap();
        match guard.get(class_name) {
            None => {
                let res = self.lookup_cache_miss(class_name, pool);
                if let Ok(classfile) = res.as_ref() {
                    guard.insert(class_name.clone(), classfile.clone());
                }
                res
            }
            Some(res) => Ok(res.clone()),
        }
    }

    pub fn lookup_cache_miss(&self, class_name: &CClassName, pool: &CompressedClassfileStringPool) -> Result<Arc<Classfile>, ClassLoadingError> {
        for x in &self.classpath_base {
            for dir_member in match x.read_dir() {
                Ok(dir) => dir,
                Err(_) => continue, //java ignores invalid classpath entries
            } {
                let dir_member = match dir_member {
                    Ok(dir_member) => dir_member,
                    Err(_) => continue,
                };
                let is_jar = dir_member.path().extension().map(|x| &x.to_string_lossy() == "jar").unwrap_or(false);
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
            if let Ok(c) = jar.lookup(pool, class_name) {
                return Result::Ok(c);
            }
        }
        for path in &self.classpath_base {
            let mut new_path = path.clone().into_path_buf();
            new_path.push(format!("{}.class", class_name.0.to_str(pool)));
            if new_path.is_file() {
                let file_read = &mut File::open(new_path).unwrap();
                let classfile = parse_class_file(file_read)?;
                return Result::Ok(Arc::new(classfile));
            }
        }
        Result::Err(ClassNotFoundException)
    }

    pub fn from_dirs(dirs: Vec<Box<Path>>) -> Self {
        Self {
            classpath_base: dirs,
            jar_cache: RwLock::new(HashMap::new()),
            class_cache: Default::default(),
        }
    }

    pub fn from_dirs_with_cache(dirs: Vec<Box<Path>>, class_cache: HashMap<CClassName, Arc<Classfile>>) -> Self {
        Self {
            classpath_base: dirs,
            jar_cache: RwLock::new(HashMap::new()),
            class_cache: RwLock::new(class_cache),
        }
    }

    pub fn classpath_string(&self) -> String {
        let mut res = String::new();
        for p in &self.classpath_base {
            res.push_str(format!("{}:", p.to_str().unwrap()).as_str());
        }
        res
    }
}