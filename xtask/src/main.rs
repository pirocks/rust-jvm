#![feature(exit_status_error)]

use std::{env, fs};
use std::collections::HashSet;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::anyhow;
use clap::{Parser};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use xshell::{cmd, Shell};

use xtask::{clean, deps, load_or_create_xtask_config, write_xtask_config, XTaskConfig};

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
            // let test_resources = PathBuf::from("/home/francis/CLionProjects/jdk8u/jdk/test");
            let javac = config.bootstrap_jdk_dir.expect("need bootstrap jdk").join("bin/javac");
            let mut command = Command::new(javac);
            let source_files = glob::glob(format!("{}/**/*.java", test_resources.to_string_lossy()).as_str())?.map(|globbed_path|{
                Ok(globbed_path?.into_os_string())
            }).collect::<Result<Vec<OsString>,anyhow::Error>>()?;
            let mut compiler_args = vec![];
            compiler_args.extend_from_slice(source_files.as_slice());
            compiler_args.push(OsString::from("-target"));
            compiler_args.push(OsString::from("1.8"));
            compiler_args.push(OsString::from("-d"));
            compiler_args.push(compilation_dir.clone().into_os_string());
            command.args(compiler_args.into_iter());
            let mut child = command.spawn()?;
            child.wait()?.exit_ok()?;
            // let sh = Shell::new()?;

            let classpath = "/home/francis/build/openjdk-debug/jdk8u/build/linux-x86_64-normal-server-slowdebug/jdk/classes /home/francis/build/openjdk-debug/jdk8u/build/linux-x86_64-normal-server-slowdebug/jdk/classes_security";
            let libjava = "/home/francis/build/openjdk-jdk8u/build/linux-x86_64-normal-server-release/jdk/lib/amd64/libjava.so";

            let exclude: HashSet<&str> = HashSet::from_iter([]);
            source_files.par_iter().try_for_each(|source_file|{
                let path_buf = PathBuf::from(source_file);
                let main = path_buf.file_stem().unwrap().to_str().unwrap();
                if !exclude.contains(main){
                    let mut args = vec![];
                    args.extend(shell_words::split(format!("run --release -- --main {} --libjava {} --classpath {} {}",main, libjava, compilation_dir.display(), classpath).as_str())?);
                    Command::new("cargo").args(args).spawn()?.wait()?;
                }
                Ok::<_, anyhow::Error>(())
            })?;
        }
        OptsInner::OpenJDKTest { .. } => {
            todo!()
        }
    }
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
