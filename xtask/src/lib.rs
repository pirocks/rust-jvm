use std::path::{Path, PathBuf};

use anyhow::anyhow;
use regex::Regex;
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

const OPENJDK_8_DOWNLOAD_URL: &str = "https://github.com/adoptium/temurin8-binaries/releases/download/jdk8u345-b01/OpenJDK8U-jdk_x64_linux_hotspot_8u345b01.tar.gz";
const DOWNLOADED_FILE_NAME: &str = "OpenJDK8U-jdk_x64_linux_hotspot_8u345b01.tar.gz";
const EXTRACTED_JDK_DIR_NAME: &str = "jdk8u345-b01";

pub fn deps(xtask_config: XTaskConfig) -> anyhow::Result<()> {
    let deps_dir = xtask_config.dep_dir;
    let deps_dir = make_deps_dir(deps_dir)?;
    let sh = Shell::new()?;
    sh.change_dir(&deps_dir);
    //todo add deps on libx11-dev libxext-dev libxrender-dev libxtst-dev libxt-dev
    if cmd!(sh, "git --version").run().is_err() {
        return Err(anyhow!("git needs to be installed"));
    }
    if cmd!(sh, "make -v").run().is_err() {
        return Err(anyhow!("make needs to be installed"));
    }
    if cmd!(sh, "g++ -v").run().is_err() {
        return Err(anyhow!("g++ needs to be installed"));
    }
    if cmd!(sh, "gcc -v").run().is_err() {
        return Err(anyhow!("gcc needs to be installed"));
    }

    let bootstrap_jdk_dir = xtask_config.bootstrap_jdk_dir;
    let bootstrap_jdk = validate_or_download_jdk(deps_dir, &sh, bootstrap_jdk_dir)?;
    //
    //todo keep track of if we have already successfully cloned and make this idempotent
    //todo make a ci idempotent tester
    cmd!(sh, "git clone --branch master --depth 1 https://github.com/pirocks/jdk8u.git").run()?;
    sh.change_dir("jdk8u");
    match bootstrap_jdk {
        None => {
            cmd!(sh,"bash configure --enable-debug --with-extra-cxxflags=\"-fpermissive\"").run()?;
        }
        Some(bootstrap_jdk) => {
            cmd!(sh,"bash configure --enable-debug --with-extra-cxxflags=\"-fpermissive\" --with-boot-jdk={bootstrap_jdk}").run()?;
        }
    }
    cmd!(sh,"make clean").run()?;
    cmd!(sh,"make DISABLE_HOTSPOT_OS_VERSION_CHECK=ok images").run()?;
    Ok(())
}

fn validate_or_download_jdk(deps_dir: PathBuf, sh: &Shell, bootstrap_jdk_dir: Option<PathBuf>) -> anyhow::Result<Option<PathBuf>> {
    if let Some(bootstrap_jdk) = bootstrap_jdk_dir {
        let java_bin_path = bootstrap_jdk.join("bin/java");
        if !validate_java_version(sh, java_bin_path.as_path()) {
            return Err(anyhow!("Default Java does not exist or is not java 7 or java 8."));
        }
        Ok(Some(bootstrap_jdk))
    } else {
        let is_okay_bootstrap_jdk = validate_java_version(sh, Path::new("java"));
        if !is_okay_bootstrap_jdk {
            eprintln!("Default Java does not exist or is not java 7 or java 8.");
            eprintln!("Downloading JDK");
            if cmd!(sh, "wget --version").run().is_err() {
                return Err(anyhow!("wget needs to be installed"));
            }
            cmd!(sh, "wget {OPENJDK_8_DOWNLOAD_URL}").run()?;
            cmd!(sh, "tar xvf {DOWNLOADED_FILE_NAME}").run()?;
            return Ok(Some(deps_dir.join(EXTRACTED_JDK_DIR_NAME)))
        }
        Ok(None)
    }
}

fn validate_java_version(sh: &Shell, java_path: &Path) -> bool {
    match cmd!(sh, "{java_path} -version").read() {
        Err(_) => {
            false
        }
        Ok(output) => {
            match output.lines().next() {
                None => {
                    false
                }
                Some(openjdk_version_line) => {
                    let version_regex = Regex::new("\"1\\.([78])\\.[0-9_]+\"").unwrap();
                    version_regex.find(openjdk_version_line).is_some()
                }
            }
        }
    }
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
    Ok(None)
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
                bootstrap_jdk_dir: None,
            })?)?;
            Ok(load_xtask_config(workspace_dir)?.unwrap())
        }
        Some(config) => {
            Ok(config)
        }
    }
}

pub fn write_xtask_config(workspace_dir: &Path, config: XTaskConfig) -> anyhow::Result<()> {
    let xtask_config_path = xtask_config_path(workspace_dir);
    std::fs::write(&xtask_config_path, ron::to_string(&config)?)?;
    Ok(())
}

#[derive(Clone, Serialize, Deserialize)]
pub struct XTaskConfig {
    pub dep_dir: PathBuf,
    pub bootstrap_jdk_dir: Option<PathBuf>,
}

impl XTaskConfig {
    pub fn rt_jar(&self) -> PathBuf {
        self.dep_dir.join("build/linux-x86_64-normal-server-fastdebug/images/j2sdk-image/jre/lib/rt.jar")
    }

    pub fn classes(&self) -> PathBuf {
        self.dep_dir.join("jdk8u/build/linux-x86_64-normal-server-fastdebug/jdk/classes/")
    }
}