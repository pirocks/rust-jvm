use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::c_void;
use std::thread::LocalKey;

use add_only_static_vec::AddOnlyVec;

use crate::{LabelName, VMExitType};

thread_local! {
pub static CODE_ADDRESS : RefCell<*mut c_void> = RefCell::new(null_mut());
}

pub struct JITState {
    code: &'static LocalKey<RefCell<*mut c_void>>,
    function_addresses: Vec<*mut c_void>,
    // indexed by method_id
    current_end: *mut c_void,
    exits: HashMap<*mut c_void, VMExitType>,
    labels: HashMap<LabelName, *mut c_void>,
}


pub struct Labeler {
    current_label: u32,
}

impl Labeler {
    pub fn new_label(&mut self) -> LabelName {
        let current_label = self.current_label.checked_add(1).unwrap();
        self.current_label = current_label;
        LabelName(current_label)
    }
}