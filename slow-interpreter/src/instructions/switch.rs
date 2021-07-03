use classfile_view::view::ptype_view::PTypeView;
use rust_jvm_common::classfile::{LookupSwitch, TableSwitch};

use crate::jvm_state::JVMState;
use crate::stack_entry::StackEntryMut;

pub fn invoke_lookupswitch(ls: &LookupSwitch, jvm: &'gc_life JVMState<'gc_life>, mut frame: StackEntryMut<'gc_life, 'l>) {
    let key = frame.pop(Some(PTypeView::IntType)).unwrap_int();
    for (candidate_key, o) in &ls.pairs {
        if *candidate_key == key {
            *frame.pc_offset_mut() = *o as i32;
            return;
        }
    }
    *frame.pc_offset_mut() = ls.default as i32;
}

pub fn tableswitch(ls: TableSwitch, jvm: &'gc_life JVMState<'gc_life>, mut frame: StackEntryMut<'gc_life, 'l>) {
    let index = frame.pop(Some(PTypeView::IntType)).unwrap_int();
    if index < ls.low || index > ls.high {
        *frame.pc_offset_mut() = ls.default as i32;
    } else {
        *frame.pc_offset_mut() = ls.offsets[(index - ls.low) as usize] as i32;
    }
}