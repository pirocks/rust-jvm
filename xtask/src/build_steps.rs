use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::path::PathBuf;

use ron::ser::{PrettyConfig, to_string_pretty};
use ron::to_string;
use serde::{Deserialize, Serialize};

//todo has libs

#[derive(Debug, Hash, Serialize, Deserialize, Eq, PartialEq)]
pub enum BuildStep {
    HasGit,
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
}

pub struct HasGitBuildStatus{

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
    fn rebuild_given_deps(&self, deps: &BuildStatus) {

    }

    fn unique_id(&self) -> String {
        to_string(self).unwrap()
    }

    fn pretty_name(&self) -> String {
        let pretty_config = PrettyConfig::new();
        to_string_pretty(self, pretty_config).unwrap()
    }

    fn deps(&self) -> HashSet<BuildStep> {
        match self {
            BuildStep::HasGit => {
                vec![]
            }
            BuildStep::HasMake => {
                vec![]
            }
            BuildStep::HasGPP => {
                vec![]
            }
            BuildStep::HasGCC => {
                vec![]
            }
            BuildStep::HasWGET => {
                vec![]
            }
            BuildStep::DownloadBootstrapJDK => {
                vec![BuildStep::HasWGET]
            }
            BuildStep::ExtractBootstrapJDK => {
                vec![BuildStep::DownloadBootstrapJDK]
            }
            BuildStep::JDK8UClone => {
                vec![BuildStep::HasGit]
            }
            BuildStep::JDK8BuildConfigure => {
                vec![BuildStep::ExtractBootstrapJDK, BuildStep::JDK8UClone]
            }
            BuildStep::MakeImages => {
                vec![BuildStep::JDK8BuildConfigure]
            }
            BuildStep::Dist => {
                vec![BuildStep::MakeImages]
            }
        }
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
