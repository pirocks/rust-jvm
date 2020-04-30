use rust_jvm_common::classfile::CPIndex;
use std::sync::Arc;
use crate::runtime_class::RuntimeClass;

use crate::java_values::JavaValue;
use std::cell::RefCell;

#[derive(Debug)]
pub struct StackEntry {
    // pub last_call_stack: Option<&StackEntry>,
    pub class_pointer: Arc<RuntimeClass>,
    pub method_i: CPIndex,

    pub local_vars: RefCell<Vec<JavaValue>>,
    pub operand_stack: RefCell<Vec<JavaValue>>,
    pub pc: RefCell<usize>,
    //the pc_offset is set by every instruction. branch instructions and others may us it to jump
    pub pc_offset: RefCell<isize>,
}

impl StackEntry {
    pub fn pop(&self) -> JavaValue {
        self.operand_stack.borrow_mut().pop().unwrap_or_else(|| {
            // let classfile = &self.class_pointer.classfile;
            // let method = &classfile.methods[self.method_i as usize];
            // dbg!(&method.method_name(&classfile));
            // dbg!(&method.code_attribute().unwrap().code);
            // dbg!(&self.pc);
            panic!()
        })
    }
    pub fn push(&self, j: JavaValue) {
        self.operand_stack.borrow_mut().push(j)
    }



   /* pub fn depth(&self) -> usize {
        match &self.last_call_stack {
            None => 0,
            Some(last) => last.depth() + 1,
        }
    }*/

    /*pub fn print_stack_trace(&self) {
        let class_file = &self.class_pointer.classfile;
        let method = &class_file.methods[self.method_i as usize];
        println!("{} {} {} {}",
                 &class_name(class_file).get_referred_name(),
                 method.method_name(class_file),
                 method.descriptor_str(class_file),
                 self.depth());
        match &self.last_call_stack {
            None => {}
            Some(last) => last.print_stack_trace(),
        }
    }*/
}

