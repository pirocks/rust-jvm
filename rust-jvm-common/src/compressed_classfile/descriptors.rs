use add_only_static_vec::{AddOnlyId, AddOnlyIdMap};

use crate::compressed_classfile::CMethodDescriptor;

pub struct CompressedMethodDescriptorsPool {
    pool: AddOnlyIdMap<CMethodDescriptor>,
}

impl CompressedMethodDescriptorsPool {
    pub fn new() -> Self {
        Self { pool: AddOnlyIdMap::new() }
    }

    pub fn add_descriptor(&self, cmd: impl Into<CMethodDescriptor>) -> ActuallyCompressedMD {
        let id = self.pool.push(cmd.into());
        ActuallyCompressedMD { id }
    }

    pub fn lookup(&self, id: ActuallyCompressedMD) -> &CMethodDescriptor {
        self.pool.lookup(id.id)
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug, PartialOrd, Ord)]
pub struct ActuallyCompressedMD {
    pub id: AddOnlyId,
}
