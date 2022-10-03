use anyhow::anyhow;
use async_trait::async_trait;
use memory_amount::MemoryAmount;
use memory_limited_executor::MemoryLimitedProcessExecutor;
use crate::build_steps::{BuildStatus, BuildStatusValidationResult, BuildStep};
use serde::{Serialize, Serializer};

pub struct HasGitBuildStatus {
//if set then we have gitdddd
}

#[derive(Serialize)]
pub struct HasGit {}

impl HasGit{
    pub async fn has_tool(&self, executor: &MemoryLimitedProcessExecutor) -> bool{
        executor.simple_process("git --version", MemoryAmount::KiloBytes(4192)).await.is_ok()
    }
}

#[async_trait]
impl HasTool for HasGit {
    async fn has_tool(&self, executor: &MemoryLimitedProcessExecutor) -> bool {
        executor.simple_process("git --version", MemoryAmount::KiloBytes(4192)).await.is_ok()
    }

    async fn notify_has_tool(deps: &BuildStatus) {
        deps.notify_has_git(HasGitBuildStatus{}).await
    }

    fn tool_name() -> String {
        "git".to_string()
    }
}


pub struct HasMakeBuildStatus {}

#[derive(Serialize)]
pub struct HasMake {}

#[async_trait]
impl HasTool for HasMake {
    async fn has_tool(&self, executor: &MemoryLimitedProcessExecutor) -> bool {
        executor.simple_process("make -v", MemoryAmount::KiloBytes(2364)).await.is_ok()
    }

    async fn notify_has_tool(deps: &BuildStatus) {
        deps.notify_has_make(HasMakeBuildStatus{}).await
    }

    fn tool_name() -> String {
        "make".to_string()
    }
}

pub struct HasGPPBuildStatus {}

#[derive(Serialize)]
pub struct HasGPP{

}

#[async_trait]
impl HasTool for HasGPP {
    async fn has_tool(&self, executor: &MemoryLimitedProcessExecutor) -> bool {
        executor.simple_process("g++ -v", MemoryAmount::KiloBytes(2800)).await.is_ok()
    }

    async fn notify_has_tool(deps: &BuildStatus) {
        deps.notify_has_gpp(HasGPPBuildStatus{}).await
    }

    fn tool_name() -> String {
        "g++".to_string()
    }
}


pub struct HasGCCBuildStatus {}

#[derive(Serialize)]
pub struct HasGCC{

}

#[async_trait]
impl HasTool for HasGCC {
    async fn has_tool(&self, executor: &MemoryLimitedProcessExecutor) -> bool {
        executor.simple_process("gcc -v", MemoryAmount::KiloBytes(2800)).await.is_ok()
    }

    async fn notify_has_tool(deps: &BuildStatus) {
        deps.notify_has_gcc(HasGCCBuildStatus{}).await
    }

    fn tool_name() -> String {
        "gcc".to_string()
    }
}



pub struct HasWgetBuildStatus {}

#[derive(Serialize)]
pub struct HasWget{

}

#[async_trait]
impl HasTool for HasWget {
    async fn has_tool(&self, executor: &MemoryLimitedProcessExecutor) -> bool {
        executor.simple_process("gcc -v", MemoryAmount::KiloBytes(5400)).await.is_ok()
    }

    async fn notify_has_tool(deps: &BuildStatus) {
        deps.notify_has_wget(HasWgetBuildStatus{}).await
    }

    fn tool_name() -> String {
        "gcc".to_string()
    }
}

#[async_trait]
impl <T: HasTool + Serialize + Send + Sync> BuildStep for T{
    async fn validate_build_status(&self, build_status: &BuildStatus) -> BuildStatusValidationResult {
        if !self.has_tool(&build_status.executor).await {
            return BuildStatusValidationResult::Error(anyhow!("{} needs to be installed", Self::tool_name()));
        }
        BuildStatusValidationResult::Okay
    }

    async fn build_deps(&self, deps: &BuildStatus) {
        //maybe require bash or something?
    }

    async fn build_given_deps(&self, build_status: &BuildStatus) -> anyhow::Result<()>{
        if !self.has_tool(&build_status.executor).await{
            return Err(anyhow!("{} needs to be installed", Self::tool_name()));
        }
        Self::notify_has_tool(build_status).await;
        Ok(())
    }
}

#[async_trait]
pub trait HasTool{
    async fn has_tool(&self, executor: &MemoryLimitedProcessExecutor) -> bool;
    async fn notify_has_tool(deps: &BuildStatus);
    fn tool_name() -> String;
}