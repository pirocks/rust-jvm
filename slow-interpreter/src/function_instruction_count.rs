use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicU64, Ordering};

use rust_jvm_common::MethodId;

pub struct FunctionInstructionExecutionCount {
    inner: RwLock<HashMap<MethodId, Arc<AtomicU64>>>,
}

impl FunctionInstructionExecutionCount {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(HashMap::new())
        }
    }

    pub fn for_function(&self, method_id: MethodId) -> FunctionExecutionCounter {
        let inner = self.inner.write().unwrap().entry(method_id).or_default().clone();
        FunctionExecutionCounter {
            inner
        }
    }

    pub fn function_instruction_count(&self, method_id: MethodId) -> u64 {
        match self.inner.read().unwrap().get(&method_id) {
            None => 0,
            Some(inner) => {
                inner.load(Ordering::SeqCst)
            }
        }
    }
}

pub struct FunctionExecutionCounter {
    inner: Arc<AtomicU64>,
}

impl FunctionExecutionCounter {
    pub fn increment(&self) {
        self.inner.fetch_add(1, Ordering::SeqCst);
    }
}
