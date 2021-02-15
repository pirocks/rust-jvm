use core::fmt;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;
use std::hash::{Hash, Hasher};

use classfile_parser::ClassfileParsingError;
use rust_jvm_common::classnames::ClassName;

use crate::view::ptype_view::ReferenceTypeView;

pub trait LivePoolGetter {
    fn elem_type(&self, idx: usize) -> ReferenceTypeView;
}

#[derive(Debug)]
pub enum ClassLoadingError {
    ClassNotFoundException,
    ClassFileInvalid(ClassfileParsingError),
    // ClassFormatError , UnsupportedClassVersionError
    ClassVerificationError,// java.lang.VerifyError
}

impl From<ClassfileParsingError> for ClassLoadingError {
    fn from(error: ClassfileParsingError) -> Self {
        ClassLoadingError::ClassFileInvalid(error)
    }
}

impl Display for ClassLoadingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "ClassNotFoundException")
    }
}

impl std::error::Error for ClassLoadingError {}


pub type LoaderIndex = usize;

#[derive(Debug)]
#[derive(Eq)]
#[derive(Clone, Hash, Copy)]
pub enum LoaderName {
    UserDefinedLoader(LoaderIndex),
    BootstrapLoader,
}

impl PartialEq for LoaderName {
    fn eq(&self, other: &LoaderName) -> bool {
        match self {
            LoaderName::BootstrapLoader => match other {
                LoaderName::BootstrapLoader => true,
                LoaderName::UserDefinedLoader(_) => false
            },

            LoaderName::UserDefinedLoader(idx) => match other {
                LoaderName::UserDefinedLoader(other_idx) => other_idx == idx,
                LoaderName::BootstrapLoader => false
            }
        }
    }
}


impl Display for LoaderName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LoaderName::BootstrapLoader => {
                write!(f, "<bl>")
            }
            LoaderName::UserDefinedLoader(idx) => {
                write!(f, "{}", *idx)
            }
        }
    }
}

// pub trait Loader {
//     fn find_loaded_class(&self, name: &ClassName) -> Option<Arc<ClassView>>;
//     fn initiating_loader_of(&self, class: &ClassName) -> bool;
//     todo File will have to be a much more general array of bytes
// fn find_representation_of(&self, class: &ClassName) -> Result<File, ClassLoadingError>;
// fn load_class(&self, class: &ClassName, live_pool_getter: Arc<dyn LivePoolGetter>) -> Result<Arc<ClassView>, ClassLoadingError>;
// fn name(&self) -> LoaderName;


//pre loading parses the class file but does not verify
// fn pre_load(&self, name: &ClassName) -> Result<Arc<ClassView>, ClassLoadingError>;
// fn add_pre_loaded(&self, name: &ClassName, classfile: &Arc<Classfile>);
// }

//todo Loading Constraints

pub const BOOTSTRAP_LOADER_NAME: &str = "bl";


// pub struct Classes {
//     //todo what about loaders with the same name? is that even possible?
//     classes: RwLock<HashMap<ClassName, HashMap<LoaderName, Arc<Classfile>>>>,
//     classpath_lookup: Box<dyn LookupInClassPath>,
// }

// pub trait LookupInClassPath {
//     fn lookup(&self, class_name: &ClassName) -> Result<Arc<Classfile>, ClassLoadingError>;
// }
//
// impl Classes {
//     pub fn class_loaded_by(&self, class: &ClassName, loader: &LoaderName) -> bool {
//         match self.classes.read().unwrap().get(class) {
//             None => false,
//             Some(class) => {
//                 class.contains_key(loader)
//             }
//         }
//     }
//
//     pub fn pre_load(&self, class: ClassName, loader: LoaderName) -> Result<Arc<Classfile>, ClassLoadingError> {
//         let mut guard = self.classes.write().unwrap();
//         let class_entry = guard.entry(class.clone()).or_insert(HashMap::new());
//         match class_entry.get(&loader) {
//             None => { self.classpath_lookup.lookup(&class) }
//             Some(class_file) => Ok(class_file.clone())
//         }
//     }
// }


// pub struct EmptyLoader {}

// pub type LoaderArc = Arc<dyn Loader + Sync + Send>;

// impl Loader for EmptyLoader {
//     fn find_loaded_class(&self, _name: &ClassName) -> Option<Arc<ClassView>> {
//         unimplemented!()
//     }
//
//     fn initiating_loader_of(&self, _class: &ClassName) -> bool {
//         unimplemented!()
//     }
//
//     fn find_representation_of(&self, _class: &ClassName) -> Result<File, ClassLoadingError> {
//         unimplemented!()
//     }
//
//     fn load_class(&self, _self_arc: LoaderArc, _class: &ClassName, _bl: LoaderArc, _live_pool_getter: Arc<dyn LivePoolGetter>) -> Result<Arc<ClassView>, ClassLoadingError> {
//         unimplemented!()
//     }
//
//     fn name(&self) -> LoaderName {
//         unimplemented!()
//     }
//
//     fn pre_load(&self, _name: &ClassName) -> Result<Arc<ClassView>, ClassLoadingError> {
//         unimplemented!()
//     }
//
//     fn add_pre_loaded(&self, _name: &ClassName, _classfile: &Arc<Classfile>) {
//         unimplemented!()
//     }
// }


pub struct ClassWithLoader {
    pub class_name: ClassName,
    pub loader: LoaderName,
}

impl Hash for ClassWithLoader {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.class_name.hash(state);
        self.loader.hash(state);
    }
}

impl PartialEq for ClassWithLoader {
    fn eq(&self, other: &ClassWithLoader) -> bool {
        self.class_name == other.class_name && self.loader == other.loader
        // Arc::ptr_eq(&self.loader, &other.loader) //todo this hella unsafe/wrong according to clippy
    }
}

impl Clone for ClassWithLoader {
    fn clone(&self) -> Self {
        ClassWithLoader { class_name: self.class_name.clone(), loader: self.loader.clone() }
    }
}

impl Eq for ClassWithLoader {}


impl Debug for ClassWithLoader {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "<{},{}>", &self.class_name.get_referred_name(), self.loader)
    }
}
