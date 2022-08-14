use std::collections::HashMap;
use std::sync::RwLock;

use wtf8::Wtf8Buf;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct CompressedWtf8String(pub usize);

impl CompressedWtf8String {
    pub fn to_wtf8(&self, pool: &Wtf8Pool) -> Wtf8Buf{
        pool.inner.read().unwrap().indices_to_buf.get(self).unwrap().clone()
    }
}

pub struct Wtf8PoolInner {
    indices_to_buf: HashMap<CompressedWtf8String, Wtf8Buf>,
    buf_to_indices: HashMap<Wtf8Buf, CompressedWtf8String>,
}

pub struct Wtf8Pool {
    inner: RwLock<Wtf8PoolInner>,
}

#[allow(clippy::new_without_default)]
impl Wtf8Pool {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(Wtf8PoolInner {
                indices_to_buf: Default::default(),
                buf_to_indices: Default::default()
            })
        }
    }

    pub fn add_entry(&self, wtf8: impl Into<Wtf8Buf>) -> CompressedWtf8String {
        let wtf8 = wtf8.into();
        let mut guard = self.inner.write().unwrap();
        if let Some(res) =  guard.buf_to_indices.get(&wtf8).cloned(){
            return res
        }
        let index = guard.indices_to_buf.len();
        let res = CompressedWtf8String(index);
        guard.indices_to_buf.insert(res, wtf8.clone());
        guard.buf_to_indices.insert(wtf8, res);
        res
    }
}

