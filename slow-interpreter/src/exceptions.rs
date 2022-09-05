use core::fmt::{Debug, Formatter};
use crate::java::lang::throwable::Throwable;

pub struct WasException<'gc>{
    pub exception_obj: Throwable<'gc>
}


impl Debug for WasException<'_>{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        todo!()
    }
}