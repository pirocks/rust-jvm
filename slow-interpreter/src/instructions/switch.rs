use rust_jvm_common::classfile::{LookupSwitch, TableSwitch};

use crate::stack_entry::StackEntryMut;
use crate::StackEntry;

pub fn invoke_lookupswitch(ls: &LookupSwitch, mut frame: StackEntryMut) {
    let key = frame.pop().unwrap_int();
    for (candidate_key, o) in &ls.pairs {
        if *candidate_key == key {
            *frame.pc_offset_mut() = *o as isize;
            return;
        }
    }
    *frame.pc_offset_mut() = ls.default as isize;
}

pub fn tableswitch(ls: TableSwitch, mut frame: StackEntryMut) {
    let index = frame.pop().unwrap_int();
    if index < ls.low || index > ls.high {
        *frame.pc_offset_mut() = ls.default as isize;
    } else {
        *frame.pc_offset_mut() = ls.offsets[(index - ls.low) as usize] as isize;
    }
}