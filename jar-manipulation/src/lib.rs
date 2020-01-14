use rust_jvm_common::classnames::ClassName;
use std::path::Path;
use zip::ZipArchive;
use rust_jvm_common::classfile::Classfile;
use std::sync::Arc;
use std::fs::File;
use std::error::Error;
use std::fmt::Formatter;
use std::fmt;

pub struct JarHandle {
    path: Box<Path>,
    zip_archive: ZipArchive<File>,//todo what if loaded from something other than file?
}

#[derive(Debug)]
pub struct NoClassFoundInJarError{}

impl std::fmt::Display for NoClassFoundInJarError{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        unimplemented!()
    }
}

impl Error for NoClassFoundInJarError{

}

impl JarHandle {
    fn new(path: Box<Path>) -> Result<JarHandle,Box<dyn Error>> {
        let f= File::open(&path)?;
        let zip_archive = zip::ZipArchive::new(f)?;
        Result::Ok(JarHandle { path, zip_archive })
    }

    fn lookup(&mut self, class_name: ClassName) -> Result<Arc<Classfile>,Box<dyn Error>>{
        let lookup_res = &self.zip_archive.by_name(class_name.get_referred_name().as_str())?;
        if lookup_res.is_file(){
            unimplemented!()
        }else {
            Result::Err(Box::new(NoClassFoundInJarError{}))
        }

    }
}