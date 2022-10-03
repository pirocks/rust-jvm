use std::path::PathBuf;
use std::sync::Arc;
use async_trait::async_trait;

use ron::ser::{PrettyConfig, to_string_pretty};
use ron::to_string;
use serde::{Serialize};
use tokio::sync::RwLock;
use memory_limited_executor::MemoryLimitedProcessExecutor;
use crate::build_steps::has_tool::{HasGCCBuildStatus, HasGitBuildStatus, HasGPPBuildStatus, HasMakeBuildStatus, HasWgetBuildStatus};

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

pub mod has_tool;
pub mod download_bootstrap_jdk;

pub enum BuildStatusValidationResult{
    NeedsRebuild,
    Okay,
    Error(anyhow::Error)
}

pub struct BuildStatus{
    build_env: BuildEnv,
    executor: MemoryLimitedProcessExecutor,
    has_git: RwLock<Option<Arc<HasGitBuildStatus>>>,
    has_make: RwLock<Option<Arc<HasMakeBuildStatus>>>,
    has_gpp: RwLock<Option<Arc<HasGPPBuildStatus>>>,
    has_gcc: RwLock<Option<Arc<HasGCCBuildStatus>>>,
    has_wget: RwLock<Option<Arc<HasWgetBuildStatus>>>,
}

impl BuildStatus{
    pub async fn notify_has_git(&self, has_git: HasGitBuildStatus){
        *self.has_git.write().await = Some(Arc::new(has_git));
    }

    pub async fn has_git(&self) -> Option<Arc<HasGitBuildStatus>>{
        self.has_git.read().await.clone()
    }


    pub async fn notify_has_make(&self, has_make: HasMakeBuildStatus){
        *self.has_make.write().await = Some(Arc::new(has_make));
    }

    pub async fn has_make(&self) -> Option<Arc<HasMakeBuildStatus>>{
        self.has_make.read().await.clone()
    }

    pub async fn notify_has_gpp(&self, has_gpp: HasGPPBuildStatus){
        *self.has_gpp.write().await = Some(Arc::new(has_gpp));
    }

    pub async fn has_gpp(&self) -> Option<Arc<HasGPPBuildStatus>>{
        self.has_gpp.read().await.clone()
    }

    pub async fn notify_has_gcc(&self, has_gcc: HasGCCBuildStatus){
        *self.has_gcc.write().await = Some(Arc::new(has_gcc));
    }

    pub async fn has_gcc(&self) -> Option<Arc<HasGCCBuildStatus>>{
        self.has_gcc.read().await.clone()
    }

    pub async fn notify_has_wget(&self, has_wget: HasWgetBuildStatus){
        *self.has_wget.write().await = Some(Arc::new(has_wget));
    }

    pub async fn has_wget(&self) -> Option<Arc<HasWgetBuildStatus>>{
        self.has_wget.read().await.clone()
    }
}

#[async_trait]
pub trait BuildStep : Serialize{
    async fn validate_build_status(&self, build_status: &BuildStatus) -> BuildStatusValidationResult;

    async fn build_deps(&self, deps: &BuildStatus);

    async fn build_given_deps(&self, deps: &BuildStatus) -> anyhow::Result<()>;

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

pub fn get_to_step(build_env: &BuildEnv, step: !/*&dyn BuildStep*/) {
    todo!()
}
