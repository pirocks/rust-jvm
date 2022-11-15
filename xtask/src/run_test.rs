use std::{fs, io};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use openjdk_test_parser::ParsedOpenJDKTest;
use thiserror::Error;
use openjdk_test_parser::parse::FileType;
use crate::file_hash::{rebuild_if_file_changed, RebuildIfError, should_rebuild, write_rebuild_if};
use crate::java_compilation::JavaCLocation;


#[derive(Error, Debug)]
pub enum TestRunError {
    #[error("Execution of this kind of test not implemented yet")]
    ExecutionNotImplemented,
    #[error(transparent)]
    IO(#[from] io::Error),
    #[error(transparent)]
    ReBuildIf(#[from] RebuildIfError),
}

pub enum TestResult {
    Success {},
    Error {},
}


pub fn run_parsed(parsed: &ParsedOpenJDKTest, test_base_dir: PathBuf, compilation_base_dir: PathBuf, javac: JavaCLocation) -> Result<TestResult, TestRunError> {
    match parsed {
        ParsedOpenJDKTest::Test {
            file_type,
            defining_file_path,
            requires,
            run,
            comment,
            build,
            library,
            key,
            modules,
            compile,
            ignore,
            clean,
            ..
        } => {
            if requires.is_some() {
                return Err(TestRunError::ExecutionNotImplemented);
            }
            if run.is_some() {
                return Err(TestRunError::ExecutionNotImplemented);
            }
            if comment.is_some() {
                return Err(TestRunError::ExecutionNotImplemented);
            }
            if build.is_some() {
                return Err(TestRunError::ExecutionNotImplemented);
            }
            if library.is_some() {
                return Err(TestRunError::ExecutionNotImplemented);
            }
            if key.is_some() {
                return Err(TestRunError::ExecutionNotImplemented);
            }
            if modules.is_some() {
                return Err(TestRunError::ExecutionNotImplemented);
            }
            if compile.is_some() {
                return Err(TestRunError::ExecutionNotImplemented);
            }
            if ignore.is_some() {
                return Err(TestRunError::ExecutionNotImplemented);
            }
            if clean.is_some() {
                return Err(TestRunError::ExecutionNotImplemented);
            }
            match file_type {
                FileType::Java => {
                    run_javac_with_rebuild_if(javac,defining_file_path, test_base_dir, compilation_base_dir)
                }
                FileType::Bash => {
                    return Err(TestRunError::ExecutionNotImplemented);
                }
                FileType::Html => {
                    return Err(TestRunError::ExecutionNotImplemented);
                }
            }
        }
    }
}


pub fn run_javac_with_rebuild_if(javac: JavaCLocation, to_compile_java_file: impl AsRef<Path>, test_base_dir: PathBuf, compilation_base_dir: PathBuf) -> Result<TestResult,TestRunError>{
    dbg!(to_compile_java_file.as_ref());
    assert!(to_compile_java_file.as_ref().starts_with(test_base_dir.clone()));
    let file_relative_path = to_compile_java_file.as_ref().strip_prefix(test_base_dir).unwrap();
    let java_file_name = file_relative_path.file_name().unwrap();
    let java_file_dir = file_relative_path.parent().unwrap();
    let compilation_target_dir = compilation_base_dir.join(java_file_dir);
    if !compilation_target_dir.exists() {
        fs::create_dir_all(&compilation_target_dir).unwrap();
    }
    let expected_output_file = compilation_target_dir.join(java_file_name).with_extension("class");
    let rebuild_if_file = expected_output_file.with_extension("rebuildif");
    if should_rebuild(&rebuild_if_file)? {
        let mut javac_command = Command::new(javac.0);
        let mut child = javac_command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .arg("-target").arg("1.8")
            .arg("-g")
            .arg("-sourcepath").arg(to_compile_java_file.as_ref().parent().unwrap())
            .arg("-d").arg(compilation_target_dir.as_path())
            .arg(to_compile_java_file.as_ref())
            .spawn()?;
        let output = child.wait_with_output().unwrap();
        if output.status.success() {
            write_rebuild_if(rebuild_if_file.as_path(),&rebuild_if_file_changed(to_compile_java_file.as_ref())?)?;
            return Ok(TestResult::Success {})
        } else {
            return Ok(TestResult::Error {})
        }
    }
    Ok(TestResult::Success {})
}