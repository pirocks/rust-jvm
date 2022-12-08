use std::{fs, io};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ron::error::SpannedError;

#[derive(Error, Debug)]
pub enum RebuildIfError {
    #[error(transparent)]
    IO(#[from]io::Error),
    #[error(transparent)]
    Ron(#[from]ron::Error),
    #[error(transparent)]
    RonSpanned(#[from]SpannedError)
}

pub fn get_file_hash(path: impl AsRef<Path>) -> Result<String, RebuildIfError> {
    let file_bytes = fs::read(path.as_ref())?;
    let sha_sum = sha256::digest(file_bytes.as_slice());
    Ok(sha_sum)
}

#[derive(Serialize, Deserialize)]
pub enum RebuildIf {
    Any {
        inner: Vec<RebuildIf>
    },
    All {
        inner: Vec<RebuildIf>
    },
    FileChanged {
        file_path: PathBuf,
        expected_hash: String,
    },
}

impl RebuildIf {
    pub fn should_rebuild(&self) -> Result<bool, RebuildIfError> {
        Ok(match self {
            RebuildIf::Any { inner } => {
                inner.iter().map(|inner| inner.should_rebuild()).collect::<Result<Vec<_>, _>>()?.into_iter().any(|x| x)
            }
            RebuildIf::All { inner } => {
                inner.iter().map(|inner| inner.should_rebuild()).collect::<Result<Vec<_>, _>>()?.into_iter().all(|x| x)
            }
            RebuildIf::FileChanged { file_path, expected_hash } => {
                let actual_hash = get_file_hash(file_path)?;
                &actual_hash != expected_hash
            }
        })
    }
}

pub fn rebuild_if_file_changed(file: impl AsRef<Path>) -> Result<RebuildIf, RebuildIfError> {
    let expected_hash = get_file_hash(file.as_ref())?;
    Ok(RebuildIf::FileChanged { file_path: file.as_ref().to_path_buf(), expected_hash })
}

pub fn should_rebuild(rebuild_if_file: impl AsRef<Path>) -> Result<bool, RebuildIfError> {
    if !rebuild_if_file.as_ref().exists() {
        return Ok(true)
    }
    let rebuild_if: RebuildIf = ron::from_str(fs::read_to_string(rebuild_if_file.as_ref())?.as_str())?;
    rebuild_if.should_rebuild()
}

pub fn write_rebuild_if(rebuild_if_file: impl AsRef<Path>, rebuild_if: &RebuildIf) -> Result<(), RebuildIfError> {
    fs::write(rebuild_if_file, ron::to_string(rebuild_if)?)?;
    Ok(())
}