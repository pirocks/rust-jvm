use std::borrow::Borrow;
use std::collections::HashSet;
use std::ops::Deref;
use std::sync::Arc;

pub type StringPoolEntry = String;

pub struct StringPool {
    pub entries: HashSet<Arc<StringPoolEntry>>,
}

impl StringPool {
    pub fn get_or_add(&mut self, s: String) -> Arc<StringPoolEntry> {
        match self.entries.borrow().get(&s) {
            None => {
                let string_arc = Arc::new(s);
                self.entries.insert(string_arc.clone());
                string_arc
            }
            Some(res) => res.clone(),
        }
    }

    pub fn garbage_collect(&mut self) {
        let mut to_remove = vec![];
        for s in &self.entries {
            if Arc::strong_count(s) == 1 {
                to_remove.push(s.clone());
            }
        }
        for x in to_remove {
            self.entries.remove(x.deref());
        }
    }
}
