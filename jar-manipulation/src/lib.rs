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
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::CompressedClassfileStringPool;

#[derive(Debug)]
pub struct JarHandle<R: Read + io::Seek> {
    pub path: Box<Path>,
    pub zip_archive: ZipArchive<R>,
}

#[derive(Debug)]
pub struct NoClassFoundInJarError {}

impl fmt::Display for NoClassFoundInJarError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{:?}", self)
    }
}

impl Error for NoClassFoundInJarError {}

impl JarHandle<File> {
    pub fn new(path: Box<Path>) -> Result<JarHandle<File>, Box<dyn Error>> {
        let f = File::open(&path)?;
        let zip_archive = zip::ZipArchive::new(f)?;
        Ok(JarHandle { path, zip_archive })
    }

    pub fn lookup(&mut self, pool: &CompressedClassfileStringPool, class_name: &CClassName) -> Result<Arc<Classfile>, Box<dyn Error>> {
        let lookup_res = &mut self.zip_archive.by_name(format!("{}.class", class_name.0.to_str(pool)).as_str())?;
        if lookup_res.is_file() {
            Result::Ok(Arc::new(parse_class_file(lookup_res)?))
        } else {
            Result::Err(Box::new(NoClassFoundInJarError {}))
        }
    }
}