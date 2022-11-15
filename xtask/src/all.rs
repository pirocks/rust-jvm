use std::cell::OnceCell;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use rayon::prelude::IntoParallelRefIterator;
use openjdk_test_parser::parse::{parse_test_file, TestParseError};
use crate::java_compilation::javac_location;
use crate::load_or_create_xtask_config;
use rayon::iter::ParallelIterator;
use tokio::time::Instant;
use openjdk_test_parser::ParsedOpenJDKTest;
use crate::run_test::{run_parsed, TestResult, TestRunError};

pub fn all_tests(workspace_dir: &PathBuf) -> anyhow::Result<()> {
    let config = load_or_create_xtask_config(workspace_dir)?;
    let compilation_dir = config.dep_dir.join("compiled_test_classes");
    if !compilation_dir.exists() {
        fs::create_dir(&compilation_dir)?;
    }
    let test_files_base = config.dep_dir.join("jdk8u/jdk/test");
    let javac = javac_location(&config);
    let java_file_paths = get_java_files(test_files_base.clone())?;
    let summary = Summary::new();
    let parsed_tests = parse_test_files_with_summary(java_file_paths, &summary);
    let test_execution_results = parsed_tests.par_iter().map(|parsed| {
        run_parsed(parsed,test_files_base.clone(), compilation_dir.clone(), javac.clone())
    }).collect::<Vec<_>>();
    summary.sink_test_results(test_execution_results.as_slice());
    todo!();
}

fn get_java_files(test_resources_base: PathBuf) -> anyhow::Result<Vec<PathBuf>> {
    let mut java_file_paths = vec![];
    for glob_result in glob::glob(format!("{}/**/*.java", test_resources_base.to_string_lossy()).as_str())? {
        let path = glob_result?.canonicalize()?;
        java_file_paths.push(path);
    }
    Ok(java_file_paths)
}

fn parse_test_files_with_summary(java_file_paths: Vec<PathBuf>, summary: &Summary) -> Vec<ParsedOpenJDKTest> {
    let parse_before = Instant::now();
    let parsed_tests = java_file_paths.par_iter().flat_map(|path| {
        match parse_test_file(path.clone()) {
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
    }).collect::<Vec<_>>();
    let parse_after = Instant::now();
    summary.sink_parse_time(parse_before, parse_after);
    summary.sink_num_parsed(parsed_tests.as_slice());
    parsed_tests
}

pub struct Summary {
    parse_fail: AtomicU64,
    parse_time: Mutex<OnceCell<f64>>,
    num_parsed: AtomicU64,
    num_compiled: AtomicU64,
    num_compile_failure: AtomicU64,
    num_not_implemented: AtomicU64
}

impl Summary {
    pub fn new() -> Self {
        Self {
            parse_fail: AtomicU64::new(0),
            parse_time: Mutex::new(OnceCell::new()),
            num_parsed: AtomicU64::new(0),
            num_compiled: AtomicU64::new(0),
            num_compile_failure: AtomicU64::new(0),
            num_not_implemented: AtomicU64::new(0)
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
        for test_result in test_results {
            match test_result {
                Ok(test_result) => {
                    match test_result {
                        TestResult::Success { .. } => {
                            self.num_compiled.fetch_add(1, Ordering::SeqCst);
                        }
                        TestResult::Error { .. } => {
                            self.num_compile_failure.fetch_add(1, Ordering::SeqCst);
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
                    }
                }
            }
        }
    }
}


impl Drop for Summary {
    fn drop(&mut self) {
        println!("Parsed Files: {}", self.num_parsed.load(Ordering::SeqCst));
        println!("Parse Time: {}", self.parse_time.lock().unwrap().get().unwrap());
        println!("Parse Failures: {}", self.parse_fail.load(Ordering::SeqCst));
        println!("Num Compilations: {}", self.num_compiled.load(Ordering::SeqCst));
        println!("Num Compilation Failures: {}", self.num_compile_failure.load(Ordering::SeqCst));
        println!("Num Not Implemented: {}", self.num_compile_failure.load(Ordering::SeqCst));
    }
}



