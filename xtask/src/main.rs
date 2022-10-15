#![feature(exit_status_error)]

use std::{env, fs};
use std::collections::HashSet;
use std::ffi::{OsStr};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::anyhow;
use clap::{Parser};
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use xshell::{cmd, Shell};

use xtask::{clean, deps, load_or_create_xtask_config, write_xtask_config, XTaskConfig};
use xtask::java_compilation::{compile, CompiledClass, javac_location};

#[derive(Parser)]
pub struct OptsOuter {
    #[clap(subcommand)]
    xtask: OptsInner,
}

#[derive(Parser)]
pub enum OptsInner {
    #[clap(about = "builds standard library and libjava.so deps")]
    Deps {
        other_dir: Option<PathBuf>,
        bootstrap_jdk: Option<PathBuf>,
    },
    #[clap(about = "set new dep dir")]
    SetDepDir {
        dep_dir: PathBuf
    },
    #[clap(about = "set new bootstrap jdk")]
    SetBootstrapJDK {
        bootstrap_jdk: PathBuf
    },
    #[clap(about = "cleans deps dir")]
    Clean {},
    #[clap(about = "create dist")]
    Dist {},
    #[clap(about = "run tests")]
    Test {},
    #[clap(about = "run openjdk tests")]
    OpenJDKTest {}
}

fn change_config_option(workspace_dir: &Path, changer: impl FnOnce(&mut XTaskConfig)) -> anyhow::Result<()> {
    let mut config = load_or_create_xtask_config(workspace_dir)?;
    changer(&mut config);
    write_xtask_config(workspace_dir, config)?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let opts: OptsOuter = OptsOuter::parse();
    let workspace_dir = workspace_dir();
    let workspace_dir = &workspace_dir;
    match opts.xtask {
        OptsInner::Deps { other_dir, bootstrap_jdk } => {
            let mut config = load_or_create_xtask_config(workspace_dir)?;
            if let Some(other_dir) = other_dir {
                config.dep_dir = other_dir;
                config.bootstrap_jdk_dir = bootstrap_jdk;
                write_xtask_config(workspace_dir, config.clone())?;
            }
            deps(config)?;
        }
        OptsInner::Clean { .. } => {
            let config = load_or_create_xtask_config(workspace_dir)?;
            clean(workspace_dir, config)?;
        }
        OptsInner::SetDepDir { dep_dir } => {
            change_config_option(&workspace_dir, |config| {
                config.dep_dir = dep_dir
            })?;
        }
        OptsInner::SetBootstrapJDK { bootstrap_jdk } => {
            change_config_option(&workspace_dir, |config| {
                config.bootstrap_jdk_dir = Some(bootstrap_jdk)
            })?;
        }
        OptsInner::Dist {} => {
            let config = load_or_create_xtask_config(workspace_dir)?;
            let release_target_dir = workspace_dir.join("target/release");
            let libjvm_so = release_target_dir.join("deps/libjvm.so");
            let java_executable = release_target_dir.join("java");
            if !java_executable.exists() {
                return Err(anyhow!("Need to build java"));
            }
            if !libjvm_so.exists() {
                return Err(anyhow!("Need to build libjvm.so"));
            }
            let dep_dir = config.dep_dir.clone();
            let jdk_dir = dep_dir.join("jdk8u/build/linux-x86_64-normal-server-fastdebug/jdk");
            let copied_jdk_dir = dep_dir.join("dist");
            fs::create_dir_all(&copied_jdk_dir)?;
            fs::create_dir_all(&copied_jdk_dir.join("bin"))?;
            let sh = Shell::new()?;
            generic_copy(&sh, jdk_dir.join("classes"), copied_jdk_dir.join("classes"))?;
            generic_copy(&sh, jdk_dir.join("classes_security"), copied_jdk_dir.join("classes_security"))?;
            generic_copy(&sh, jdk_dir.join("lib"), copied_jdk_dir.join("lib"))?;
            generic_copy(&sh, &java_executable, copied_jdk_dir.join("bin").join("java"))?;
            generic_copy(&sh, &libjvm_so, copied_jdk_dir.join("bin").join("libjvm.so"))?;
        }
        OptsInner::Test { } => {
            let config = load_or_create_xtask_config(workspace_dir)?;
            let compilation_dir = config.dep_dir.join("compiled_test_classes");
            if !compilation_dir.exists(){
                fs::create_dir(&compilation_dir)?;
            }
            let test_resources = workspace_dir.join("tests/resource_classes");
            let javac = javac_location(&config);
            let source_files = glob::glob(format!("{}/**/*.java", test_resources.to_string_lossy()).as_str())?.map(|globbed_path|{
                Ok(globbed_path?)
            }).collect::<Result<Vec<PathBuf>,anyhow::Error>>()?;
            let class_files = source_files.into_iter().map(|source_file|{
                Ok(compile(&javac, source_file.as_path(), compilation_dir.as_path())?)
            }).collect::<anyhow::Result<Vec<CompiledClass>>>()?;
            let exclude: HashSet<String> = HashSet::from_iter([]);
            run_classes(compilation_dir, class_files, exclude)?;
        }
        OptsInner::OpenJDKTest {  } => {
            let config = load_or_create_xtask_config(workspace_dir)?;
            let compilation_dir = config.dep_dir.join("compiled_test_classes");
            if !compilation_dir.exists(){
                fs::create_dir(&compilation_dir)?;
            }
            let test_resources_base = config.dep_dir.join("jdk8u/jdk/test");
            let javac = javac_location(&config);
            // let to
            let classes = vec![
                "java/lang/Boolean/Factory",
                "java/lang/Boolean/GetBoolean",
                "java/lang/Boolean/MakeBooleanComparable",
                "java/lang/Boolean/ParseBoolean",
                "java/lang/Byte/Decode",
                // "java/lang/Character/TestIsJavaIdentifierMethods", //needs perf, specifically big loop needs compilation
                "java/lang/Long/BitTwiddle",
                // "java/lang/Long/Decode", // needs working npe
                "java/lang/Long/GetLong",
                "java/lang/Long/ParsingTest",
                // "java/lang/Long/Unsigned", //needs working division by zero
                // "java/lang/Thread/GenerifyStackTraces", //needs impl dump threads
                // "java/lang/Thread/HoldsLock",// needs impl holds lock
                "java/lang/Thread/MainThreadTest",
                // "java/lang/Thread/ITLConstructor",// seems to deadlock needs fix
                "java/lang/Compare",
                "java/lang/HashCode",
                "java/lang/ToString",
                //"java/util/AbstractCollection/ToArrayTest", // needs array store exception checking on arrays to be implemented.
                "java/util/AbstractCollection/ToString",
                // "java/util/AbstractList/CheckForComodification", //ignored by openjdk b/c openjdk std is broken
                "java/util/AbstractList/FailFastIterator",
                "java/util/AbstractList/HasNextAfterException",
                "java/util/AbstractMap/AbstractMapClone",
                // "java/util/AbstractMap/Equals", // causes an expected npe need to implement npe throwing
                "java/util/AbstractMap/SimpleEntries",
                "java/util/AbstractMap/ToString",
                "java/util/AbstractSequentialList/AddAll",
                "java/util/ArrayList/AddAll",
                "java/util/ArrayList/Bug6533203",
                "java/util/ArrayList/EnsureCapacity",
                // "java/util/ArrayList/IteratorMicroBenchmark", //takes long af. though I guess I should fix perf bug
                // "java/util/ArrayList/RangeCheckMicroBenchmark"//takes long af. though I guess I should fix perf bug

            ];
            let class_files = classes.into_par_iter().map(|class|{
                test_resources_base.join(format!("{}.java",class))
            }).map(|source_file|{
                Ok(compile(&javac, source_file.as_path(), compilation_dir.as_path())?)
            }).collect::<anyhow::Result<Vec<CompiledClass>>>()?;
            let exclude: HashSet<String> = HashSet::from_iter([]);
            run_classes(compilation_dir, class_files, exclude)?;
        }
    }
    Ok(())
}

fn run_classes(compilation_dir: PathBuf, class_files: Vec<CompiledClass>, exclude: HashSet<String>) -> anyhow::Result<()> {
    let classpath = "/home/francis/build/openjdk-debug/jdk8u/build/linux-x86_64-normal-server-slowdebug/jdk/classes /home/francis/build/openjdk-debug/jdk8u/build/linux-x86_64-normal-server-slowdebug/jdk/classes_security";
    let libjava = "/home/francis/build/openjdk-jdk8u/build/linux-x86_64-normal-server-release/jdk/lib/amd64/libjava.so";

    class_files.into_iter().try_for_each(|main| {
        if !exclude.contains(&main.name()) {
            let mut args = vec![];
            args.extend(shell_words::split(format!("run --release -- --main {} --libjava {} --classpath {} {}", main.name(), libjava, compilation_dir.display(), classpath).as_str())?);
            Command::new("cargo").args(args).spawn()?.wait()?;
        }
        Ok::<_, anyhow::Error>(())
    })?;
    Ok(())
}

fn generic_copy<From: AsRef<OsStr>, To: AsRef<OsStr>>(sh: &Shell, from: From, to: To) -> anyhow::Result<()> {
    cmd!(sh, "cp -rf {from} {to}").run()?;
    Ok(())
}

fn xtask_dir() -> PathBuf {
    PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set?"))
}

fn workspace_dir() -> PathBuf {
    xtask_dir().parent().unwrap().to_path_buf()
}
