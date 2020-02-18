use std::sync::Arc;
use std::fmt::{Display, Debug, Formatter, Error};
use std::fmt;
use std::fs::File;
use rust_jvm_common::classnames::ClassName;
use std::hash::{Hash, Hasher};

//#[derive(Hash)]
pub struct ClassWithLoader {
    pub class_name: ClassName,
    pub loader: LoaderArc,
}

impl Hash for ClassWithLoader {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.class_name.hash(state);
        self.loader.name().hash(state);
    }
}

impl PartialEq for ClassWithLoader {
    fn eq(&self, other: &ClassWithLoader) -> bool {
        self.class_name == other.class_name &&
            Arc::ptr_eq(&self.loader, &other.loader)
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
        write!(f, "<{},{}>", &self.class_name.get_referred_name(), self.loader.name())
    }
}



#[derive(Debug)]
#[derive(Hash)]
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

#[derive(Debug)]
pub enum ClassLoadingError {
    ClassNotFoundException,
}

impl Display for ClassLoadingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "ClassNotFoundException")
    }
}

pub trait Loader {
    fn initiating_loader_of(&self, class: &ClassName) -> bool;
    //todo File will have to be a much more general array of bytes
    fn find_representation_of(&self, class: &ClassName) -> Result<File, ClassLoadingError>;
    fn load_class(&self, self_arc: LoaderArc, class: &ClassName, bl: LoaderArc) -> Result<Arc<stage2_common::classfile::Classfile>, ClassLoadingError>;
    fn name(&self) -> LoaderName;


    //pre loading parses the class file but does not verify
    fn pre_load(&self, self_arc: LoaderArc, name: &ClassName) -> Result<Arc<stage2_common::classfile::Classfile>, ClassLoadingError>;
}




//todo Loading Constraints

pub const BOOTSTRAP_LOADER_NAME: &str = "bl";

pub type LoaderArc = Arc<dyn Loader + Sync + Send>;
