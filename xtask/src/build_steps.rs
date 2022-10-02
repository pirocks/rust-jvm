use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::path::PathBuf;
use anyhow::anyhow;

use ron::ser::{PrettyConfig, to_string_pretty};
use ron::to_string;
use serde::{Deserialize, Serialize};
use xshell::cmd;

//todo has libs

/*    HasGit,
    HasMake,
    HasGPP,
    HasGCC,
    HasWGET,
    DownloadBootstrapJDK,
    ExtractBootstrapJDK,
    JDK8UClone,
    JDK8BuildConfigure,
    MakeImages,
    Dist,
*/
pub struct HasGitBuildStatus{

}

pub struct HasGit{

}

impl HasTool for HasGit{

}

impl BuildStep for HasGit {
    type AssociatedBuildStatus = HasToolBuildStatus;

    fn validate_build_status(&self, build_status: &BuildStatus) {
        if cmd!(sh, "git --version").run().is_err() {
            return Err(anyhow!("git needs to be installed"));
        }
    }
}

pub trait HasTool: BuildStep{




    fn build_deps(&self, deps: &BuildStatus) {
        todo!()
    }

    fn rebuild_given_deps(&self, deps: &BuildStatus) {
        todo!()
    }
}

pub struct HasToolBuildStatus{

}

pub struct BuildStatus{
    build_env: BuildEnv,
    has_git: Option<HasGitBuildStatus>
}

impl BuildStatus{
    pub fn has_git(&self) -> HasGitBuildStatus{

    }
}

pub trait BuildStep{
    type AssociatedBuildStatus;

    fn validate_build_status(&self, build_status: &BuildStatus);

    fn build_deps(&self, deps: &BuildStatus);

    fn rebuild_given_deps(&self, deps: &BuildStatus);

    fn unique_id(&self) -> String {
        to_string(self).unwrap()
    }

    fn pretty_name(&self) -> String {
        let pretty_config = PrettyConfig::new();
        to_string_pretty(self, pretty_config).unwrap()
    }
}

pub struct BuildStepIdentifier {}

pub struct BuildEnv {
    repo_dir: PathBuf,
    build_dir: PathBuf,
}

pub fn get_to_step(build_env: &BuildEnv, step: BuildStep) {
    todo!()
}
