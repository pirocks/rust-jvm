use std::any::Any;
use std::ptr::NonNull;
use std::sync::atomic::AtomicBool;

use nonnull_const::NonNullConst;

use crate::SignalAccessibleJavaStackData;

pub enum RemoteQuery {
    GetGuestFrameStackInstructionPointer,
    GC,
}

#[derive(Debug)]
pub enum RemoteQuerySafe<'l> {
    GetGuestFrameStackInstructionPointer {
        answer: &'l mut Option<GetGuestFrameStackInstructionPointer>,
        answer_written: &'l AtomicBool,
    },
    RestartFromGetGuestFrameStackInstructionPointer,
    GC,
}

impl RemoteQuerySafe<'_> {
    pub fn to_remote_query_unsafe(self, signal_safe_data: &SignalAccessibleJavaStackData) -> RemoteQueryUnsafe {
        let signal_safe_data = NonNullConst::new(signal_safe_data as *const SignalAccessibleJavaStackData).unwrap();
        match self {
            RemoteQuerySafe::GetGuestFrameStackInstructionPointer { answer, answer_written } => {
                RemoteQueryUnsafe {
                    query_type: RemoteQueryInternalType::GetGuestFrameStackInstructionPointer {
                        answer: NonNull::new(answer as *mut _).unwrap(),
                        answer_written: NonNullConst::new(answer_written as *const _).unwrap(),
                    },
                    signal_safe_data,
                }
            }
            RemoteQuerySafe::GC => {
                todo!()
            }
            RemoteQuerySafe::RestartFromGetGuestFrameStackInstructionPointer => {
                RemoteQueryUnsafe{
                    query_type: RemoteQueryInternalType::RestartFromGetGuestFrameStackInstructionPointer,
                    signal_safe_data
                }
            }
        }
    }

    pub fn wait_for_next_signal(&self) -> bool {
        match self {
            RemoteQuerySafe::GetGuestFrameStackInstructionPointer { .. } => {
                true
            }
            RemoteQuerySafe::GC => {
                todo!()
            }
            RemoteQuerySafe::RestartFromGetGuestFrameStackInstructionPointer => {
                todo!()
            }
        }
    }
}

#[derive(Debug)]
pub struct RemoteQueryUnsafe {
    query_type: RemoteQueryInternalType,
    signal_safe_data: NonNullConst<SignalAccessibleJavaStackData>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum RemoteQueryInternalType {
    GetGuestFrameStackInstructionPointer {
        answer: NonNull<Option<GetGuestFrameStackInstructionPointer>>,
        answer_written: NonNullConst<AtomicBool>,
    },
    RestartFromGetGuestFrameStackInstructionPointer,
    GC,
}

impl RemoteQueryUnsafe {
    pub fn query_type(&self) -> &RemoteQueryInternalType{
        &self.query_type
    }

    pub fn to_remote_query<'l>(&self) -> RemoteQuerySafe<'l> {
        match self.query_type {
            RemoteQueryInternalType::GetGuestFrameStackInstructionPointer { mut answer, answer_written } => {
                unsafe { RemoteQuerySafe::GetGuestFrameStackInstructionPointer { answer: answer.as_mut(), answer_written: answer_written.as_ref() } }
            }
            RemoteQueryInternalType::GC => {
                todo!()
            }
            RemoteQueryInternalType::RestartFromGetGuestFrameStackInstructionPointer => {
                RemoteQuerySafe::RestartFromGetGuestFrameStackInstructionPointer
            }
        }
    }

    pub fn signal_safe_data<'l>(&self) -> &'l SignalAccessibleJavaStackData{
        unsafe { self.signal_safe_data.as_ref() }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum GetGuestFrameStackInstructionPointer {
    InGuest {
        rbp: u64,
        rsp: u64,
        rip: u64,
    },
    InVM {
        rbp: u64,
        rsp: u64,
        rip: u64,
    },
    Transitioning {},
    FrameBeingCreated {
        rbp: u64,
        rsp: u64,
        rip: u64,
    },
}


pub enum RemoteQueryAnswerInternal {
    GetGuestFrameStackInstructionPointer {
        answer: GetGuestFrameStackInstructionPointer,
    },
    Panic(Box<dyn Any + Send>),
    Empty,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum RemoteQueryAnswer {
    GetGuestFrameStackInstructionPointer(GetGuestFrameStackInstructionPointer),
}
