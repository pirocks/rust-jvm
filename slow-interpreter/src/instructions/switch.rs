use rust_jvm_common::classfile::{LookupSwitch, TableSwitch};
use std::rc::Rc;
use runtime_common::StackEntry;

pub fn invoke_lookupswitch(ls: &LookupSwitch, frame: &Rc<StackEntry>) {
    let key = frame.pop().unwrap_int();
    for (candidate_key,o) in &ls.pairs{
        if *candidate_key == key{
            frame.pc_offset.replace(*o as isize);
            return;
        }
    }
    frame.pc_offset.replace(ls.default as isize);
}

pub fn tableswitch(ls: TableSwitch, frame: &Rc<StackEntry>) {
    let index = frame.pop().unwrap_int();
    if index < ls.low || index > ls.high{
        frame.pc_offset.replace(ls.default as isize);
    }else {
        frame.pc_offset.replace(ls.offsets[index - ls.low]);
    }
}