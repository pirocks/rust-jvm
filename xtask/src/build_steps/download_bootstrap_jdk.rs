use async_trait::async_trait;
use crate::build_steps::{BuildStatus, BuildStatusValidationResult, BuildStep};
use serde::Serialize;


#[derive(Serialize)]
pub struct DownloadBootstrapJDK{

}

#[async_trait]
impl BuildStep for DownloadBootstrapJDK{
    #[allow(unused)]
    async fn validate_build_status(&self, build_status: &BuildStatus) -> BuildStatusValidationResult {
        todo!()
    }

    #[allow(unused)]
    async fn build_deps(&self, deps: &BuildStatus) {
        todo!()
    }

    #[allow(unused)]
    async fn build_given_deps(&self, deps: &BuildStatus) -> anyhow::Result<()> {
        todo!()
    }
}
