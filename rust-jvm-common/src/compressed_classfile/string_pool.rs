use add_only_static_vec::{AddOnlyId, AddOnlyIdMap};
use crate::compressed_classfile::names;

pub struct CompressedClassfileStringPool {
    pool: AddOnlyIdMap<String>,
}

static mut ONLY_ONE: bool = false;

impl CompressedClassfileStringPool {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        unsafe {
            if ONLY_ONE {
                panic!("should only be one CompressedClassfileStringPool")
            }
            ONLY_ONE = true;
        }
        let pool: AddOnlyIdMap<String> = AddOnlyIdMap::new();
        names::add_all_names(&pool);
        Self { pool }
    }

    pub fn add_name(&self, str: impl Into<String>, is_class_name: bool) -> CompressedClassfileString {
        let string = str.into();
        if is_class_name && string.starts_with('[') {
            dbg!(&string);
            todo!();
        }
        let id = self.pool.push(string);
        CompressedClassfileString { id }
    }

    pub fn lookup(&self, id: CompressedClassfileString) -> &String {
        self.pool.lookup(id.id)
    }
}

pub type CCString = CompressedClassfileString;

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug, PartialOrd, Ord)]
pub struct CompressedClassfileString {
    pub id: AddOnlyId,
}

impl CompressedClassfileString {
    pub fn to_str(&self, pool: &CompressedClassfileStringPool) -> String {
        pool.pool.lookup(self.id).to_string()
    }
}
