#![feature(exit_status_error)]

use std::{env, fs};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::anyhow;
use clap::{Parser};
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
    Test {}
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
            let test_resources = workspace_dir.join("tests/resource_classes");
            let javac = config.bootstrap_jdk_dir.expect("need bootstrap jdk").join("bin/javac");
            let mut command = Command::new(javac);
            let mut source_files = glob::glob(format!("{}/**/*.java", test_resources.to_string_lossy()).as_str())?.map(|globbed_path|{
                Ok(globbed_path?.into_os_string())
            }).collect::<Result<Vec<OsString>,anyhow::Error>>()?;
            source_files.push(OsString::from("-target"));
            source_files.push(OsString::from("1.8"));
            source_files.push(OsString::from("-d"));
            let target_dir = workspace_dir.join("target");
            source_files.push(target_dir.into_os_string());
            command.args(source_files.into_iter());
            let mut child = command.spawn()?;
            child.wait()?.exit_ok()?;
            let sh = Shell::new()?;

            cmd!(sh,"cargo run --release --   --main net.minecraft.server.MinecraftServer  --libjava /home/francis/build/openjdk-jdk8u/build/linux-x86_64-normal-server-release/jdk/lib/amd64/libjava.so --classpath /home/francis/Clion/rust-jvm/resources/test /home/francis/build/openjdk-debug/jdk8u/build/linux-x86_64-normal-server-slowdebug/jdk/classes /home/francis/build/openjdk-debug/jdk8u/build/linux-x86_64-normal-server-slowdebug/jdk/classes_security").run()?;

            todo!();
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
