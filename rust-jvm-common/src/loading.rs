use core::fmt;
use std::sync::Arc;
use crate::classfile::Classfile;
use crate::classnames::ClassName;
use std::fs::File;
use std::fmt::Display;
use std::fmt::Debug;
use std::error::Error;
use std::fmt::Formatter;

#[derive(Debug)]
pub enum ClassLoadingError {
    ClassNotFoundException,
}

impl Display for ClassLoadingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "ClassNotFoundException")
    }
}

impl Error for ClassLoadingError {}


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
    fn load_class(&self, self_arc: LoaderArc, class: &ClassName, bl: LoaderArc) -> Result<Arc<Classfile>, ClassLoadingError>;
    fn name(&self) -> LoaderName;


    //pre loading parses the class file but does not verify
    fn pre_load(&self, self_arc: LoaderArc, name: &ClassName) -> Result<Arc<Classfile>, ClassLoadingError>;
}

//todo Loading Constraints

pub const BOOTSTRAP_LOADER_NAME: &str = "bl";

pub struct EmptyLoader {}

pub type LoaderArc = Arc<dyn Loader + Sync + Send>;

impl Loader for EmptyLoader {
    fn initiating_loader_of(&self, _class: &ClassName) -> bool {
        unimplemented!()
    }

    fn find_representation_of(&self, _class: &ClassName) -> Result<File, ClassLoadingError> {
        unimplemented!()
    }

    fn load_class(&self, _self_arc: LoaderArc, _class: &ClassName, _bl: LoaderArc) -> Result<Arc<Classfile>, ClassLoadingError> {
        unimplemented!()
    }

    fn name(&self) -> LoaderName {
        unimplemented!()
    }

    fn pre_load(&self, _self_arc: LoaderArc, _name: &ClassName) -> Result<Arc<Classfile>, ClassLoadingError> {
        unimplemented!()
    }
}