use std::{fs, io};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;
use shell_words::ParseError;
use openjdk_test_parser::ParsedOpenJDKTest;
use thiserror::Error;
use wait_timeout::ChildExt;
use openjdk_test_parser::parse::FileType;
use crate::file_hash::{rebuild_if_file_changed, RebuildIfError, should_rebuild, write_rebuild_if};
use crate::java_compilation::JavaCLocation;


#[derive(Error, Debug)]
pub enum TestRunError {
    #[error("Execution of this kind of test not implemented yet")]
    ExecutionNotImplemented,
    #[error("timeout")]
    Timeout,
    #[error("jvm exited unsuccessfully")]
    ProcessFailure,
    #[error(transparent)]
    IO(#[from] io::Error),
    #[error(transparent)]
    ReBuildIf(#[from] RebuildIfError),
    #[error(transparent)]
    ShellWordsParseError(#[from] ParseError)
}

pub enum TestResult {
    Success {},
    Error {},
}

pub enum TestCompilationResult {
    Success {
        compiled_file: PathBuf
    },
    Error {},
}


pub fn run_parsed(parsed: &ParsedOpenJDKTest, test_base_dir: PathBuf, compilation_base_dir: PathBuf, javac: JavaCLocation, jdk_dir: PathBuf) -> Result<TestResult, TestRunError> {
    return match parsed {
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
                    match run_javac_with_rebuild_if(javac, defining_file_path, test_base_dir, compilation_base_dir)? {
                        TestCompilationResult::Success { compiled_file } => {
                            let libjava = jdk_dir.join("build/linux-x86_64-normal-server-fastdebug/jdk/lib/amd64/libjava.so");
                            let main = compiled_file.file_stem().unwrap();
                            let classpath_1 = format!("{}/build/linux-x86_64-normal-server-fastdebug/jdk/classes", jdk_dir.display());
                            let classpath_2 = format!("{}/build/linux-x86_64-normal-server-fastdebug/jdk/classes_security", jdk_dir.display());
                            let mut child = Command::new("cargo")
                                .arg("run").arg("--release").arg("--")
                                .arg("--main").arg(main.to_str().unwrap())
                                .arg("--libjava").arg(libjava)
                                .arg("--classpath")
                                .arg(compiled_file.parent().unwrap())
                                .arg(classpath_1.as_str())
                                .arg(classpath_2.as_str())
                                .spawn()?;
                            match child.wait_timeout(Duration::from_secs(10))?{
                                None => {
                                    child.kill()?;
                                    let _ = child.wait()?;
                                    return Err(TestRunError::Timeout)
                                }
                                Some(status) => {
                                    if !status.success() {
                                        return Err(TestRunError::ProcessFailure)
                                    }
                                }
                            };
                            Ok(TestResult::Success {})
                        }
                        TestCompilationResult::Error { .. } => {
                            Ok(TestResult::Error {})
                        }
                    }
                }
                FileType::Bash => {
                    Err(TestRunError::ExecutionNotImplemented)
                }
                FileType::Html => {
                    Err(TestRunError::ExecutionNotImplemented)
                }
            }
        }
    };
}


pub fn run_javac_with_rebuild_if(javac: JavaCLocation, to_compile_java_file: impl AsRef<Path>, test_base_dir: PathBuf, compilation_base_dir: PathBuf) -> Result<TestCompilationResult, TestRunError> {
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
        let child = javac_command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .arg("-target").arg("1.8")
            .arg("-g")
            .arg("-sourcepath").arg(to_compile_java_file.as_ref().parent().unwrap())
            .arg("-d").arg(compilation_target_dir.as_path())
            .arg(to_compile_java_file.as_ref())
            .spawn()?;
        let output = child.wait_with_output().unwrap();
        return if output.status.success() {
            write_rebuild_if(rebuild_if_file.as_path(), &rebuild_if_file_changed(to_compile_java_file.as_ref())?)?;
            Ok(TestCompilationResult::Success { compiled_file: expected_output_file })
        } else {
            Ok(TestCompilationResult::Error {})
        };
    }
    Ok(TestCompilationResult::Success { compiled_file: expected_output_file })
}