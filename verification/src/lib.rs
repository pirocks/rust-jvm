extern crate log;
extern crate simple_logger;
extern crate elapsed;

use elapsed::{measure_time, ElapsedDuration};
use crate::verifier::class_is_type_safe;
use crate::verifier::Frame;
use crate::verifier::TypeSafetyError;
use std::collections::vec_deque::VecDeque;
use classfile_view::vtype::VType;
use classfile_view::view::ClassView;
use classfile_view::loading::{LoaderArc, ClassWithLoader, LivePoolGetter};
use std::time::Duration;
use std::sync::Arc;


pub mod verifier;

static mut TOTAL_VERIFICATION: Duration = Duration::from_micros(0);

pub fn verify(vf: &VerifierContext, to_verify: ClassView, loader: LoaderArc) -> Result<(), TypeSafetyError> {
    let (time, res) = measure_time(|| match class_is_type_safe(vf, &ClassWithLoader {
        class_name: to_verify.name(),
        loader,
    }) {
        Ok(_) => Result::Ok(()),
        Err(err) => {
            match err {
                TypeSafetyError::NotSafe(s) => {
                    dbg!(s);
                    unimplemented!()
                }
                TypeSafetyError::NeedToLoad(_) => unimplemented!(),
            }
        }
    });
    unsafe {
        TOTAL_VERIFICATION = TOTAL_VERIFICATION.checked_add(time.duration()).unwrap();
        println!("Total: {}", ElapsedDuration::new(TOTAL_VERIFICATION));
    }
    println!("Verification Time: {}", time);
    res
}

#[derive(Debug)]
pub struct StackMap {
    pub offset: usize,
    pub map_frame: Frame,
}

pub struct VerifierContext {
    pub live_pool_getter: Arc<dyn LivePoolGetter>,
    pub bootstrap_loader: LoaderArc
}

impl Clone for VerifierContext {
    fn clone(&self) -> Self {
        VerifierContext { live_pool_getter: self.live_pool_getter.clone(), bootstrap_loader: self.bootstrap_loader.clone() }
    }
}

#[derive(Eq, Debug)]
pub struct OperandStack {
    data: VecDeque<VType>
}

impl Clone for OperandStack {
    fn clone(&self) -> Self {
        OperandStack {
            data: self.data.clone()
        }
    }
}

impl PartialEq for OperandStack {
    fn eq(&self, _other: &OperandStack) -> bool {
        unimplemented!()
    }
}


impl OperandStack {
    pub fn operand_push(&mut self, type_: VType) {
        self.data.push_front(type_);
    }

    pub fn operand_pop(&mut self) -> VType {
        self.data.pop_front().unwrap()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn peek(&self) -> VType {
        self.data.front().unwrap().clone()
    }

    pub fn new_prolog_display_order(types: &Vec<VType>) -> OperandStack {
        let mut o = OperandStack::empty();
        for type_ in types {
            o.operand_push(type_.clone())
        }
        o
    }

    pub fn new_reverse_display_order(_types: &Vec<VType>) -> OperandStack {
        unimplemented!()
    }

    pub fn empty() -> OperandStack {
        OperandStack { data: VecDeque::new() }
    }

    pub fn iter(&self) -> std::collections::vec_deque::Iter<'_, VType> {
        self.data.iter()
    }

    pub fn substitute(&mut self, old: &VType, new: &VType) {
        for entry in &mut self.data {
            if entry == old {
                *entry = new.clone();
            }
        }
    }
}


