use core::fmt;
use std::sync::{RwLock, Arc};
use std::collections::HashMap;
use crate::classfile::Classfile;

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




#[derive(Debug)]
pub struct Loader {
    //todo look at what spec has to say about this in more detail
    pub loaded: RwLock<HashMap<ClassEntry, Arc<Classfile>>>,
    pub loading: RwLock<HashMap<ClassEntry, Arc<Classfile>>>,
    pub name: String,
}
