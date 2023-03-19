use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::executor::Executor;
use crate::fullnames::FullName;

pub struct BuilderConfig {
    targets: Vec<BuilderTarget>,
}

impl BuilderConfig {
    pub fn new() -> Self{
        Self{
            targets: vec![],
        }
    }

    pub fn add_target(&mut self,
                      full_name: FullName,
                      init_rebuild_conditions: impl FnOnce(&mut HashMap<FullName, HashSet<RebuildCondition>>),
                      init_artifacts: impl FnOnce(&mut HashSet<PathBuf>),
    ) {
        let mut dep_rebuild_conditions = HashMap::new();
        init_rebuild_conditions(&mut dep_rebuild_conditions);
        let mut artifacts_from_base = HashSet::new();
        init_artifacts(&mut artifacts_from_base);
        let target = BuilderTarget {
            full_name,
            dep_rebuild_conditions,
            artifacts_from_base,
            build_func: Box::new(|_|{
                todo!()
            }),
        };
        self.targets.push(target)
    }
}


pub enum RebuildCondition {
    ArtifactMatchesStoredHash {
        path_from_base: PathBuf,
    },
    Any {
        conditions: HashSet<RebuildCondition>
    }
}


pub enum Artifacts{
    Absolute(HashSet<PathBuf>),
    Relative(HashSet<PathBuf>)
}

pub struct BuilderTarget {
    full_name: FullName,
    dep_rebuild_conditions: HashMap<FullName, HashSet<RebuildCondition>>,
    artifacts_from_base: HashSet<PathBuf>,
    build_func: Box<dyn FnOnce(PathBuf) -> HashSet<PathBuf>>
}

pub trait BuildFunction{
    fn build(executor: &dyn Executor, path: &Path) -> Artifacts;
}

#[cfg(test)]
pub mod test {
    use crate::builder_config::{BuilderConfig};
    use crate::fullnames::FullNameBuilder;

    #[test]
    pub fn use_test() {
        let full_name_builder = FullNameBuilder::new("uk.co.pirocks.jvm");
        let bootstrap_jdk_download_name = full_name_builder.new_name("jdk download").unwrap();
        let mut build_config = BuilderConfig::new();
        build_config.add_target(bootstrap_jdk_download_name,|_|{},|artifact_paths|{})

    }
}
