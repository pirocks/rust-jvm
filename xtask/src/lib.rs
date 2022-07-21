use std::path::{Path, PathBuf};

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use xshell::{cmd, Shell};


fn default_deps_dir(workspace_dir: &Path) -> PathBuf {
    workspace_dir.join("deps")
}

pub fn clean(workspace_dir: &Path, config: XTaskConfig) -> anyhow::Result<()> {
    let sh = Shell::new()?;
    sh.change_dir(workspace_dir);
    sh.cmd("rm").arg("-rf").arg(&config.dep_dir).run()?;
    Ok(())
}

fn make_deps_dir(dir: PathBuf) -> anyhow::Result<PathBuf> {
    if !dir.exists() {
        std::fs::create_dir_all(&dir)?;
    }

    Ok(dir)
}

pub fn deps(dep_dir: PathBuf) -> anyhow::Result<()> {
    let deps_dir = make_deps_dir(dep_dir)?;
    let sh = Shell::new()?;
    sh.change_dir(deps_dir);
    if let Err(_) = cmd!(sh, "git --version").run() {
        return Err(anyhow!("git needs to be installed"));
    }
    if let Err(_) = cmd!(sh, "make -v").run() {
        return Err(anyhow!("make needs to be installed"));
    }
    if let Err(_) = cmd!(sh, "g++ -v").run() {
        return Err(anyhow!("g++ needs to be installed"));
    }
    if let Err(_) = cmd!(sh, "gcc -v").run() {
        return Err(anyhow!("gcc needs to be installed"));
    }
    //todo keep track of if we have already successfully cloned and make this idempotent
    cmd!(sh, "git clone --branch master --depth 1 https://github.com/pirocks/jdk8u.git").run()?;
    sh.change_dir("jdk8u");
    cmd!(sh,"bash configure --enable-debug --with-extra-cxxflags=\"-fpermissive\" ").run()?;
    cmd!(sh,"make clean").run()?;
    cmd!(sh,"make DISABLE_HOTSPOT_OS_VERSION_CHECK=ok jdk").run()?;
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub enum DepsStatus {
    Cloned,
    Chmod,
    GetSource,
    Configure,
    MakeClean,
    MakeJdk,
}

pub fn load_xtask_config(workspace_dir: &Path) -> anyhow::Result<Option<XTaskConfig>> {
    let xtask_config_path = workspace_dir.join("xtask.config");
    if xtask_config_path.exists() {
        let xtask_config_string = std::fs::read_to_string(&xtask_config_path)?;
        return Ok(Some(ron::from_str(xtask_config_string.as_str())?));
    }
    return Ok(None);
}

fn xtask_config_path(workspace_dir: &Path) -> PathBuf {
    workspace_dir.join("xtask.config")
}

pub fn load_or_create_xtask_config(workspace_dir: &Path) -> anyhow::Result<XTaskConfig> {
    match load_xtask_config(workspace_dir)? {
        None => {
            let xtask_config_path = xtask_config_path(workspace_dir);
            std::fs::write(&xtask_config_path, ron::to_string(&XTaskConfig {
                dep_dir: default_deps_dir(workspace_dir),
                build_jdk_dir: None
            })?)?;
            return Ok(load_xtask_config(workspace_dir)?.unwrap());
        }
        Some(config) => {
            return Ok(config);
        }
    }
}

pub fn write_xtask_config(workspace_dir: &Path,config: XTaskConfig) -> anyhow::Result<()> {
    let xtask_config_path = xtask_config_path(workspace_dir);
    std::fs::write(&xtask_config_path, ron::to_string(&config)?)?;
    Ok(())
}

#[derive(Clone, Serialize, Deserialize)]
pub struct XTaskConfig {
    pub dep_dir: PathBuf,
    pub build_jdk_dir: Option<PathBuf>
}