use std::env;
use std::path::{Path, PathBuf};
use serde::Serialize;
use serde::Deserialize;

use anyhow::anyhow;
use xshell::{cmd, Shell};

fn main() -> anyhow::Result<()> {
    let task = env::args().nth(1);
    match task.as_ref().map(|it| it.as_str()) {
        Some("deps") => deps()?,
        Some("clean") => clean()?,
        _ => print_help(),
    };
    Ok(())
}

fn xtask_dir() -> PathBuf {
    PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set?"))
}

fn workspace_dir() -> PathBuf {
    xtask_dir().parent().unwrap().to_path_buf()
}

fn deps_dir() -> PathBuf {
    workspace_dir().join("deps")
}

fn clean() -> anyhow::Result<()> {
    let sh = Shell::new()?;
    sh.change_dir(workspace_dir());
    cmd!(sh, "rm -rf deps").run()?;
    Ok(())
}

fn make_deps_dir() -> anyhow::Result<PathBuf> {
    if !deps_dir().exists() {
        std::fs::create_dir_all(&deps_dir())?;
    }
    Ok(deps_dir())
}

fn deps() -> anyhow::Result<()> {
    let workspace_dir = workspace_dir();
    let deps_dir = make_deps_dir()?;
    let sh = Shell::new()?;
    sh.change_dir(deps_dir);
    if let Err(_) = cmd!(sh, "git --version").run() {
        return Err(anyhow!("git needs to be installed"));
    }
    if let Err(_) = cmd!(sh, "make -v").run() {
        return Err(anyhow!("make needs to be installed"));
    }
    //todo keep track of if we have already successfully cloned and make this idempotent
    cmd!(sh, "git clone --branch master --depth 1 https://github.com/pirocks/jdk8u.git").run()?;
    sh.change_dir("jdk8u");
    cmd!(sh,"bash configure --enable-debug --with-extra-cxxflags=\"-fpermissive\" ").run()?;
    cmd!(sh,"make clean").run()?;
    cmd!(sh,"make DISABLE_HOTSPOT_OS_VERSION_CHECK=ok jdk").run()?;
}

#[derive(Serialize,Deserialize)]
pub enum DepsStatus{
    Cloned,
    Chmod,
    GetSource,
    Configure,
    MakeClean,
    MakeJdk
}

fn print_help() {
    eprintln!(
        "Tasks:
deps           builds standard library and libjava.so deps
clean          cleans deps dir
"
    )
}