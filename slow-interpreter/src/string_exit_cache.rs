use std::collections::HashMap;

use sketch_jvm_version_of_utf8::wtf8_pool::CompressedWtf8String;

use crate::JString;
use crate::new_java_values::allocated_objects::AllocatedHandle;

//todo these should really be combined.
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


pub struct StringInternment<'gc> {
    pub(crate) strings: HashMap<Vec<u16>, AllocatedHandle<'gc>>,
}


impl <'gc> StringInternment<'gc>{
    pub fn new() -> StringInternment<'gc> {
        Self{
            strings: HashMap::new(),
        }
    }
}
