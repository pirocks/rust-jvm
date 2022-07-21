use std::{env, fs};
use std::ffi::OsStr;
use std::path::PathBuf;
use anyhow::anyhow;

use clap::Parser;
use xshell::{cmd, Shell};
use xtask::{clean, deps, load_or_create_xtask_config, write_xtask_config};

#[derive(Parser)]
pub struct OptsOuter {
    #[clap(subcommand)]
    xtask: OptsInner,
}

#[derive(Parser)]
pub enum OptsInner {
    #[clap(about = "builds standard library and libjava.so deps")]
    Deps {
        other_dir: Option<PathBuf>
    },
    #[clap(about = "set new dep dir")]
    SetDepDir {
        dep_dir: PathBuf
    },
    #[clap(about = "cleans deps dir")]
    Clean {},
    #[clap(about = "create dist")]
    Dist {}
}

fn main() -> anyhow::Result<()> {
    let opts: OptsOuter = OptsOuter::parse();
    let workspace_dir =  workspace_dir();
    let workspace_dir =  &workspace_dir;
    match opts.xtask {
        OptsInner::Deps { other_dir } => {
            let mut config = load_or_create_xtask_config(workspace_dir)?;
            if let Some(other_dir) = other_dir {
                config.dep_dir = other_dir;
                write_xtask_config(workspace_dir,config.clone())?;
            }
            deps(config.dep_dir)?;
        }
        OptsInner::Clean { .. } => {
            let config = load_or_create_xtask_config(workspace_dir)?;
            clean(workspace_dir,config)?;
        }
        OptsInner::SetDepDir { dep_dir } => {
            let mut config = load_or_create_xtask_config(workspace_dir)?;
            config.dep_dir = dep_dir;
            write_xtask_config(workspace_dir,config)?;
        }
        OptsInner::Dist {  } => {
            let config = load_or_create_xtask_config(workspace_dir)?;
            let release_target_dir = workspace_dir.join("target/release");
            let libjvm_so = release_target_dir.join("deps/libjvm.so");
            let java_executable = release_target_dir.join("java");
            if !java_executable.exists(){
                return Err(anyhow!("Need to build java"))
            }
            if !libjvm_so.exists(){
                return Err(anyhow!("Need to build libjvm.so"))
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
    }
    Ok(())
}

fn generic_copy<From: AsRef<OsStr>, To: AsRef<OsStr>>(sh: &Shell, from: From, to: To) -> anyhow::Result<()>{
    cmd!(sh, "cp -rf {from} {to}").run()?;
    Ok(())
}

fn xtask_dir() -> PathBuf {
    PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set?"))
}

fn workspace_dir() -> PathBuf {
    xtask_dir().parent().unwrap().to_path_buf()
}
