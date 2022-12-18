use std::{fs, io};
use std::os::unix::prelude::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::{Duration, Instant};

use shell_words::ParseError;
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio::time::timeout;

use openjdk_test_parser::parse::FileType;
use openjdk_test_parser::ParsedOpenJDKTest;

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
    ShellWordsParseError(#[from] ParseError),
}

pub enum TestResult {
    Success {
        instant_before: Instant,
        instant_after: Instant,
    },
    Error {},
}

pub enum TestCompilationResult {
    Success {
        compiled_file: PathBuf
    },
    Error {},
}


pub async fn run_parsed(test_output_lock: Arc<tokio::sync::Mutex<()>>, parsed: ParsedOpenJDKTest, test_base_dir: PathBuf, compilation_base_dir: PathBuf, javac: JavaCLocation, jdk_dir: PathBuf, java_binary: PathBuf) -> Result<TestResult, TestRunError> {
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
                    match run_javac_with_rebuild_if(javac, defining_file_path.clone(), test_base_dir, compilation_base_dir).await? {
                        TestCompilationResult::Success { compiled_file } => {
                            run_test(test_output_lock, jdk_dir, java_binary, compiled_file, defining_file_path).await
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

async fn run_test(test_output_lock: Arc<tokio::sync::Mutex<()>>, jdk_dir: PathBuf, java_binary: PathBuf, compiled_file: PathBuf, source_file: PathBuf) -> Result<TestResult, TestRunError> {
    let java_home = jdk_dir.join("build/linux-x86_64-normal-server-fastdebug/jdk/");
    let main = compiled_file.file_stem().unwrap();
    let classpath_1 = format!("{}/build/linux-x86_64-normal-server-fastdebug/jdk/classes", jdk_dir.display());
    let classpath_2 = format!("{}/build/linux-x86_64-normal-server-fastdebug/jdk/classes_security", jdk_dir.display());


    let mut child = Command::new("systemd-run")
        .arg("--no-ask-password")
        .arg("--scope")
        .arg("--no-block")
        .arg("-p")
        .arg("RuntimeMaxSec=60")
        .arg("-p")
        .arg("MemoryMax=1G")
        .arg("-p")
        .arg("MemoryHigh=512M")
        .arg("-p")
        .arg("MemorySwapMax=400M")
        .arg("-p")
        .arg("ManagedOOMSwap=kill")
        .arg("-p")
        .arg("ManagedOOMMemoryPressure=kill")
        .arg("--user")
        .arg(&java_binary)
        .arg("--main").arg(main.to_str().unwrap())
        .arg("--java-home").arg(&java_home)
        .arg("--classpath")
        .arg(compiled_file.parent().unwrap())
        .arg(classpath_1.as_str())
        .arg(classpath_2.as_str())
        .arg("--properties")
        .arg(format!("test.src={}", source_file.parent().unwrap().display()))
        .arg("--properties")
        .arg(format!("test.classes={}", compiled_file.parent().unwrap().display()))
        .current_dir(compiled_file.parent().unwrap())
        .stdout(Stdio::piped()).stderr(Stdio::piped())
        .spawn()?;

    let run_string = format!("{} --main {} --java-home {} --classpath {} {} {} --properties {}",
                             java_binary.display(), main.to_str().unwrap(), java_home.display(),
                             compiled_file.parent().unwrap().display(), classpath_1.as_str(), classpath_2.as_str(),
                             format!("test.src={}", source_file.parent().unwrap().display())
    );

    let instant_before = Instant::now();
    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();
    match timeout(Duration::from_secs(59), child.wait()).await {
        Err(_) => {
            child.start_kill().unwrap();
            let mut stdout_buf = vec![];
            stdout.read_to_end(&mut stdout_buf).await.unwrap();
            let mut stderr_buf = vec![];
            stderr.read_to_end(&mut stderr_buf).await.unwrap();
            output_run_output(run_string, ExitKind::Timeout, test_output_lock, stdout_buf, stderr_buf, compiled_file).await;
            Err(TestRunError::Timeout)
        }
        Ok(status) => {
            let instant_after = Instant::now();
            let status = status?;
            let success = status.success();
            let mut stdout_buf = vec![];
            stdout.read_to_end(&mut stdout_buf).await.unwrap();
            let mut stderr_buf = vec![];
            stderr.read_to_end(&mut stderr_buf).await.unwrap();
            if !success {
                output_run_output(run_string, ExitKind::ExitCode(status.into_raw()), test_output_lock, stdout_buf, stderr_buf, compiled_file).await;
                return Err(TestRunError::ProcessFailure);
            }
            Ok(TestResult::Success {
                instant_before,
                instant_after,
            })
        }
    }
}

#[derive(Debug)]
pub enum ExitKind {
    Success,
    ExitCode(i32),
    Timeout,
}

async fn output_run_output(run_string: String, exit_kind: ExitKind, output_lock: Arc<tokio::sync::Mutex<()>>, stdout_buf: Vec<u8>, stderr_buf: Vec<u8>, file: PathBuf) {
    let guard = output_lock.lock().await;
    println!("Output from:{}", file.display());
    println!("{}", run_string);
    println!("{:?}", exit_kind);
    println!("Stdout:");
    tokio::io::stdout().write_all(stdout_buf.as_ref()).await.unwrap();
    println!("Stderr:");
    tokio::io::stdout().write_all(stderr_buf.as_ref()).await.unwrap();
    tokio::io::stdout().flush().await.unwrap();
    drop(guard);
}


pub async fn run_javac_with_rebuild_if(javac: JavaCLocation, to_compile_java_file: impl AsRef<Path>, test_base_dir: PathBuf, compilation_base_dir: PathBuf) -> Result<TestCompilationResult, TestRunError> {
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
        let output = child.wait_with_output().await.unwrap();
        return if output.status.success() {
            write_rebuild_if(rebuild_if_file.as_path(), &rebuild_if_file_changed(to_compile_java_file.as_ref())?)?;
            Ok(TestCompilationResult::Success { compiled_file: expected_output_file })
        } else {
            Ok(TestCompilationResult::Error {})
        };
    }
    Ok(TestCompilationResult::Success { compiled_file: expected_output_file })
}