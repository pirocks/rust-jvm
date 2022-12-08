use std::collections::HashMap;

use sketch_jvm_version_of_utf8::wtf8_pool::CompressedWtf8String;

use crate::JString;

pub struct StringExitCache<'gc> {
    inner: HashMap<CompressedWtf8String, JString<'gc>>,
}

impl<'gc> StringExitCache<'gc> {
    pub fn new() -> Self {
        Self {
            inner: Default::default()
        }
    }

    pub fn lookup(&self, wtf8: CompressedWtf8String) -> Option<&JString<'gc>> {
        self.inner.get(&wtf8)
    }

    pub fn register_entry(&mut self, wtf8: CompressedWtf8String, jstring: JString<'gc>) {
        self.inner.insert(wtf8, jstring);
    }
}
