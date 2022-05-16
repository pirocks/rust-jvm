use std::collections::HashMap;
use std::hash::Hash;
use libc::c_void;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct SkipableExitID(u64);


pub struct AssemblySkipableExit {
    pub(crate) assembly_instruct_idx: usize
}

pub struct SkipableExit {
    pub(crate) jump_address: *mut c_void,
}

pub struct AssemblySkipableExits{
    pub inner: HashMap<SkipableExitID, AssemblySkipableExit>
}

impl AssemblySkipableExits{
    pub fn new() -> Self{
        Self{
            inner: HashMap::new()
        }
    }
}

pub struct SkipableExits {
    current_max_id: SkipableExitID,
    inner: HashMap<SkipableExitID, SkipableExit>,
}

impl SkipableExits {
    pub fn new() -> Self {
        Self {
            current_max_id: SkipableExitID(0),
            inner: HashMap::new(),
        }
    }

    pub fn new_id(&mut self) -> SkipableExitID {
        let id = self.current_max_id;
        self.current_max_id.0 += 1;
        id
    }

    pub fn sink_exit(&mut self, skipable_exit_id: SkipableExitID, skipable_exit: SkipableExit) {
        self.inner.insert(skipable_exit_id, skipable_exit);
    }
}