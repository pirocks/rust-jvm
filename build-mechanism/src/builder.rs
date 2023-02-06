use std::path::PathBuf;

use crate::builder_config::BuilderConfig;

pub struct Builder {
    build_dir: PathBuf,
    builder_config: BuilderConfig,
}

impl Builder {
    pub fn new(build_dir: impl Into<PathBuf>, builder_config: BuilderConfig) -> Self{
        let build_dir = build_dir.into();
        Self {
            build_dir,
            builder_config,
        }
    }
}