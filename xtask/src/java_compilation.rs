use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;
use crate::XTaskConfig;

pub struct JavaCLocation(pub PathBuf);

pub fn javac_location(config: &XTaskConfig) -> JavaCLocation{
    JavaCLocation(config.bootstrap_jdk_dir.as_ref().expect("need bootstrap jdk").join("bin/javac"))
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct CompiledClass(pub PathBuf);

impl CompiledClass{
    pub fn name(&self) -> String{
        self.0.file_prefix().unwrap().to_string_lossy().to_string()
    }
}

pub fn compile(javac_location: &JavaCLocation, to_compile: Vec<PathBuf>, compilation_target_dir: &Path) -> anyhow::Result<CompiledClass> {
    let mut command = Command::new(&javac_location.0);
    let class_name = to_compile[0].file_prefix().unwrap();
    let mut compiler_args = vec![];
    for to_compile in to_compile.iter(){
        compiler_args.push(to_compile.as_os_str());
    }
    compiler_args.push(OsStr::new("-target"));
    compiler_args.push(OsStr::new("1.8"));
    compiler_args.push(OsStr::new("-g"));
    compiler_args.push(OsStr::new("-d"));
    compiler_args.push(compilation_target_dir.as_os_str());
    command.args(compiler_args.into_iter());
    dbg!(&command);
    let mut child = command.spawn()?;
    child.wait()?.exit_ok()?;
    Ok(CompiledClass(compilation_target_dir.join(class_name).with_extension(".class")))
}