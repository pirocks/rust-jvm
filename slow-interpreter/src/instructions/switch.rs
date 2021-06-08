use classfile_view::view::ptype_view::PTypeView;
use rust_jvm_common::classfile::{LookupSwitch, TableSwitch};

use crate::stack_entry::StackEntryMut;

pub fn invoke_lookupswitch(ls: &LookupSwitch, mut frame: StackEntryMut) {
    let key = frame.pop(PTypeView::IntType).unwrap_int();
    for (candidate_key, o) in &ls.pairs {
        if *candidate_key == key {
            *frame.pc_offset_mut() = *o as i32;
            return;
        }
    }
    *frame.pc_offset_mut() = ls.default as i32;
}

pub fn tableswitch(ls: TableSwitch, mut frame: StackEntryMut) {
    let index = frame.pop(PTypeView::IntType).unwrap_int();
    if index < ls.low || index > ls.high {
        *frame.pc_offset_mut() = ls.default as i32;
    } else {
        *frame.pc_offset_mut() = ls.offsets[(index - ls.low) as usize] as i32;
    }
}