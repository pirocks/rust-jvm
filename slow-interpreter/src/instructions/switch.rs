use rust_jvm_common::classfile::LookupSwitch;
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