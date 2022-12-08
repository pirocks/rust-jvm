use std::collections::HashMap;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct OpaqueID(pub u64);

pub struct OpaqueIDInfo {
    debug_info: &'static str,
}

pub struct OpaqueIDs {
    info: HashMap<OpaqueID, OpaqueIDInfo>,
}

impl OpaqueIDs {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            info: Default::default()
        }
    }


    pub fn get_opaque_id_info(&self, opaque_id: OpaqueID) -> &OpaqueIDInfo {
        self.info.get(&opaque_id).unwrap()
    }

    pub fn new_opaque_id(&mut self, debug_info: &'static str) -> OpaqueID {
        let new_id = OpaqueID(self.info.len() as u64);
        self.info.insert(new_id, OpaqueIDInfo { debug_info });
        new_id
    }
}
