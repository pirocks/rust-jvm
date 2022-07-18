use std::env;
use std::path::PathBuf;

use clap::Parser;
use xtask::{clean, deps, load_or_create_xtask_config, write_xtask_config};

#[derive(Parser)]
pub struct OptsOuter {
    #[clap(subcommand)]
    xtask: OptsInner,
}

#[derive(Parser)]
pub enum OptsInner {
    #[clap(about = "builds standard library and libjava.so deps")]
    Deps {
        other_dir: Option<PathBuf>
    },
    #[clap(about = "set new dep dir")]
    SetDepDir {
        dep_dir: PathBuf
    },
    #[clap(about = "cleans deps dir")]
    Clean {},
}

fn main() -> anyhow::Result<()> {
    let opts: OptsOuter = OptsOuter::parse();
    let workspace_dir =  workspace_dir();
    let workspace_dir =  &workspace_dir;
    match opts.xtask {
        OptsInner::Deps { other_dir } => {
            let mut config = load_or_create_xtask_config(workspace_dir)?;
            if let Some(other_dir) = other_dir {
                config.dep_dir = other_dir;
                write_xtask_config(workspace_dir,config.clone())?;
            }
            deps(config.dep_dir)?;
        }
        OptsInner::Clean { .. } => {
            let config = load_or_create_xtask_config(workspace_dir)?;
            clean(workspace_dir,config)?;
        }
        OptsInner::SetDepDir { dep_dir } => {
            let mut config = load_or_create_xtask_config(workspace_dir)?;
            config.dep_dir = dep_dir;
            write_xtask_config(workspace_dir,config)?;
        }
    }
    Ok(())
}

fn xtask_dir() -> PathBuf {
    PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set?"))
}

fn workspace_dir() -> PathBuf {
    xtask_dir().parent().unwrap().to_path_buf()
}
