use std::cell::OnceCell;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use futures::StreamExt;
use tokio::task::spawn_blocking;

use openjdk_test_parser::parse::{parse_test_file, TestParseError};
use openjdk_test_parser::ParsedOpenJDKTest;

use crate::execution::{run_parsed, TestResult, TestRunError};
use crate::java_compilation::javac_location;
use crate::load_or_create_xtask_config;

pub async fn all_tests(workspace_dir: &PathBuf) -> anyhow::Result<()> {
    let config = load_or_create_xtask_config(workspace_dir)?;
    let jdk_dir = config.dep_dir.join("jdk8u");
    let compilation_dir = config.dep_dir.join("compiled_test_classes");
    if !compilation_dir.exists() {
        tokio::fs::create_dir(&compilation_dir).await?;
    }
    let test_files_base = config.dep_dir.join("jdk8u/jdk/test");
    let javac = javac_location(&config);
    let java_file_paths = get_java_files(test_files_base.clone()).await?;
    let summary = Summary::new();
    let parsed_tests = parse_test_files_with_summary(java_file_paths, &summary).await;
    let java_binary = build_jvm(workspace_dir).await?;
    let output_lock = Arc::new(tokio::sync::Mutex::new(()));
    let test_execution_results = futures::stream::iter(parsed_tests.into_iter().map(|parsed| {
        run_parsed(output_lock.clone(), parsed, test_files_base.clone(), compilation_dir.clone(), javac.clone(), jdk_dir.clone(), java_binary.clone())
    })).buffer_unordered(64).collect::<Vec<_>>().await;
    summary.sink_test_results(test_execution_results.as_slice());
    Ok(())
}

async fn build_jvm(workspace_dir: &PathBuf) -> anyhow::Result<PathBuf> {
    tokio::process::Command::new("cargo")
        .arg("build")
        .arg("--manifest-path").arg(workspace_dir.join("Cargo.toml"))
        .arg("--release")
        .spawn()?.wait().await?.exit_ok()?;
    Ok(workspace_dir.join("target/release/java"))
}

async fn get_java_files(test_resources_base: PathBuf) -> anyhow::Result<Vec<PathBuf>> {
    spawn_blocking(move ||{
        let mut java_file_paths = vec![];
        for glob_result in glob::glob(format!("{}/**/*.java", test_resources_base.to_string_lossy()).as_str())? {
            let path = glob_result?.canonicalize()?;
            java_file_paths.push(path);
        }
        Ok(java_file_paths)
    }).await.unwrap()
}

async fn parse_test_files_with_summary(java_file_paths: Vec<PathBuf>, summary: &Summary) -> Vec<ParsedOpenJDKTest> {
    let parse_before = Instant::now();
    let parsed_tests: Vec<ParsedOpenJDKTest> = futures::future::join_all(java_file_paths.into_iter().map(async move |path| {
        match parse_test_file(path.clone()).await {
            Ok(parsed) => {
                Some(parsed)
            }
            Err(err) => {
                match &err {
                    TestParseError::ContainsNoTest => {
                        None
                    }
                    TestParseError::IncompatibleFileType |
                    TestParseError::TokenError(_) |
                    TestParseError::IO(_) => {
                        summary.sink_parse_failure(err);
                        None
                    }
                }
            }
        }
    })).await.into_iter().flatten().collect();
    let parse_after = Instant::now();
    summary.sink_parse_time(parse_before, parse_after);
    summary.sink_num_parsed(parsed_tests.as_slice());
    parsed_tests
}

pub struct Summary {
    parse_fail: AtomicU64,
    parse_time: Mutex<OnceCell<f64>>,
    average_test_runtime: Mutex<OnceCell<f64>>,
    num_parsed: AtomicU64,
    num_success: AtomicU64,
    num_failure: AtomicU64,
    num_not_implemented: AtomicU64,
    timeout: AtomicU64,
    process_failure: AtomicU64,
}

impl Summary {
    pub fn new() -> Self {
        Self {
            parse_fail: AtomicU64::new(0),
            parse_time: Mutex::new(OnceCell::new()),
            average_test_runtime: Mutex::new(OnceCell::new()),
            num_parsed: AtomicU64::new(0),
            num_success: AtomicU64::new(0),
            num_failure: AtomicU64::new(0),
            num_not_implemented: AtomicU64::new(0),
            timeout: AtomicU64::new(0),
            process_failure: AtomicU64::new(0),
        }
    }

    pub fn sink_parse_failure(&self, _err: TestParseError) {
        self.parse_fail.fetch_add(1, Ordering::SeqCst);
    }

    pub fn sink_parse_time(&self, instant_before: Instant, instant_after: Instant) {
        self.parse_time.lock().unwrap().set(instant_after.saturating_duration_since(instant_before).as_secs_f64()).unwrap();
    }

    pub fn sink_num_parsed(&self, parsed: &[ParsedOpenJDKTest]) {
        self.num_parsed.store(parsed.len() as u64, Ordering::SeqCst);
    }

    pub fn sink_test_results(&self, test_results: &[Result<TestResult, TestRunError>]) {
        //todo cleanup
        let mut test_runtimes = vec![];
        for test_result in test_results {
            match test_result {
                Ok(test_result) => {
                    match test_result {
                        TestResult::Success { instant_before, instant_after } => {
                            test_runtimes.push(instant_after.saturating_duration_since(*instant_before).as_secs_f64());
                            self.num_success.fetch_add(1, Ordering::SeqCst);
                        }
                        TestResult::Error { .. } => {
                            self.num_failure.fetch_add(1, Ordering::SeqCst);
                        }
                    }
                }
                Err(test_error) => {
                    match test_error {
                        TestRunError::ExecutionNotImplemented => {
                            self.num_not_implemented.fetch_add(1, Ordering::SeqCst);
                        }
                        TestRunError::IO(_) => {
                            todo!()
                        }
                        TestRunError::ReBuildIf(_) => {
                            todo!()
                        }
                        TestRunError::ShellWordsParseError(_) => {
                            todo!()
                        }
                        TestRunError::Timeout => {
                            self.timeout.fetch_add(1, Ordering::SeqCst);
                        }
                        TestRunError::ProcessFailure => {
                            self.process_failure.fetch_add(1, Ordering::SeqCst);
                        }
                    }
                }
            }
        }
        let sum = test_runtimes.iter().sum::<f64>();
        let avg = sum / (test_runtimes.len() as f64);
        self.average_test_runtime.lock().unwrap().set(avg).unwrap();
    }
}


impl Drop for Summary {
    fn drop(&mut self) {
        println!("Parsed Files: {}", self.num_parsed.load(Ordering::SeqCst));
        println!("Parse Time: {}", self.parse_time.lock().unwrap().get().unwrap());
        println!("Parse Failures: {}", self.parse_fail.load(Ordering::SeqCst));
        println!("Num Successes: {}", self.num_success.load(Ordering::SeqCst));
        println!("Num Failures: {}", self.num_failure.load(Ordering::SeqCst));
        println!("Num Not Implemented: {}", self.num_not_implemented.load(Ordering::SeqCst));
        println!("Timeouts: {}", self.timeout.load(Ordering::SeqCst));
        println!("Process Failures: {}", self.process_failure.load(Ordering::SeqCst));
        println!("Average Test Time: {}", self.average_test_runtime.lock().unwrap().get().unwrap());
    }
}



