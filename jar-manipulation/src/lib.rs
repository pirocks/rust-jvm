use std::{fmt, io};
use std::error::Error;
use std::fmt::Formatter;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;

use zip::ZipArchive;

use classfile_parser::parse_class_file;
use rust_jvm_common::classfile::Classfile;
use rust_jvm_common::classnames::ClassName;

#[derive(Debug)]
pub struct JarHandle<R: Read + io::Seek> {
    pub path: Box<Path>,
    pub zip_archive: ZipArchive<R>,
}

impl Clone for JarHandle<File> {
    fn clone(&self) -> Self {
        let f = File::open(&self.path).unwrap();
        let zip_archive = zip::ZipArchive::new(f).unwrap();
        Self
        {
            path: self.path.clone(),
            zip_archive,
        }
    }
}

#[derive(Debug)]
pub struct NoClassFoundInJarError {}

impl std::fmt::Display for NoClassFoundInJarError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{:?}", self)
    }
}

impl Error for NoClassFoundInJarError {}

impl JarHandle<File> {
    pub fn new(path: Box<Path>) -> Result<JarHandle<File>, Box<dyn Error>> {
        let f = File::open(&path)?;
        let zip_archive = zip::ZipArchive::new(f)?;
        Result::Ok(JarHandle { path, zip_archive })
    }

    pub fn lookup(&mut self, class_name: &ClassName) -> Result<Arc<Classfile>, Box<dyn Error>> {
        let lookup_res = &mut self.zip_archive.by_name(format!("{}.class", class_name.get_referred_name()).as_str())?;
        if lookup_res.is_file() {
            Result::Ok(Arc::new(parse_class_file(lookup_res)?))
        } else {
            Result::Err(Box::new(NoClassFoundInJarError {}))
        }
    }
}
