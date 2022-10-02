use std::collections::HashMap;
use std::ffi::OsString;
use std::hash::Hash;
use std::io;
use std::path::PathBuf;
use std::process::ExitStatus;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::process::Child;
use tokio::sync::RwLock;
use meminfo_parser::async_current_meminfo;
use memory_amount::MemoryAmount;

pub struct RunningProcessData{
    start_data: ProcessStartData,
    child: Child
}

pub struct ProcessStartData{
    binary: OsString,
    args: Vec<OsString>,
    working_dir: Option<PathBuf>,
    env_vars: Option<HashMap<OsString,OsString>>,
    expected_max_ram: MemoryAmount
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct ProccessHandle{
    process_id: u64
}

pub struct MemoryLimitedProcessExecutorInner{
    running_processes: HashMap<ProccessHandle, Arc<RwLock<RunningProcessData>>>,
    finished_processes: HashMap<ProccessHandle, Arc<io::Result<ExitStatus>>>
}

impl MemoryLimitedProcessExecutorInner{
    fn new() -> Self{
        Self{
            running_processes: HashMap::new(),
            finished_processes: HashMap::new()
        }
    }
}

pub struct MemoryLimitedProcessExecutor{
    id_num: AtomicU64,
    inner: RwLock<MemoryLimitedProcessExecutorInner>
}

impl MemoryLimitedProcessExecutor{
    pub fn new() -> Self{
        Self{
            id_num: AtomicU64::new(0),
            inner: RwLock::new(MemoryLimitedProcessExecutorInner::new())
        }
    }

    fn new_process_handle(&self) -> ProccessHandle{
        let new_id_num = self.id_num.fetch_add(1, Ordering::SeqCst);
        ProccessHandle{
            process_id: new_id_num
        }
    }

    async fn available_ram(&self) -> MemoryAmount{
        async_current_meminfo("/proc/meminfo").await.expect("Unable to parse meminfo?").available
    }

    pub async fn submit_process(&self, start_data: ProcessStartData) -> ProccessHandle{
        let ProcessStartData{ working_dir, binary, args, env_vars, expected_max_ram } = &start_data;
        loop {
            if self.available_ram().await >= *expected_max_ram{
                break;
            }
            tokio::time::sleep(Duration::from_micros(100)).await;
        }
        let mut command = tokio::process::Command::new(binary);
        command.args(args.into_iter());
        if let Some(env_vars) = env_vars{
            command.envs(env_vars.into_iter());
        }
        if let Some(working_dir) = working_dir{
            command.current_dir(working_dir);
        }
        let mut child = command.spawn().expect("Failed to spawn?");
        let new_handle = self.new_process_handle();
        let mut running_processes_guard = self.inner.write().await;
        running_processes_guard.running_processes.insert(new_handle, Arc::new(RwLock::new(RunningProcessData{
            start_data,
            child
        })));
        new_handle
    }

    pub async fn complete_process(&self, id: ProccessHandle) -> Arc<io::Result<ExitStatus>>{
        let inner_read_guard = self.inner.read().await;
        let running_process_data = match inner_read_guard.running_processes.get(&id) {
            Some(running_process_data) => running_process_data.clone(),
            None => {
                return inner_read_guard.finished_processes.get(&id).unwrap().clone()
            },
        };
        drop(inner_read_guard);
        let wait_result = running_process_data.write().await.child.wait().await;
        let mut inner_write_guard = self.inner.write().await;
        let _ =  inner_write_guard.running_processes.remove(&id);
        let res = Arc::new(wait_result);
        inner_write_guard.finished_processes.insert(id, res.clone());
        res
    }
}

#[cfg(test)]
pub mod test{
    use std::ffi::OsString;
    use std::process::id;
    use memory_amount::MemoryAmount;
    use crate::{MemoryLimitedProcessExecutor, ProcessStartData};

    #[tokio::test]
    pub async fn test_basic() {
        let executor = MemoryLimitedProcessExecutor::new();
        let mut ids_to_wait = vec![];
        ids_to_wait.push(executor.submit_process(echo_test()).await);
        for _ in 0..1000{
            ids_to_wait.push(executor.submit_process(sleep()).await);
        }
        for id_to_wait in ids_to_wait{
            assert!(executor.complete_process(id_to_wait).await.as_ref().as_ref().unwrap().success());
        }
    }

    fn echo_test() -> ProcessStartData {
        ProcessStartData {
            binary: OsString::from("echo"),
            args: vec![OsString::from("test")],
            working_dir: None,
            env_vars: None,
            expected_max_ram: MemoryAmount::KiloBytes(1)
        }
    }

    fn sleep() -> ProcessStartData {
        ProcessStartData {
            binary: OsString::from("sleep"),
            args: vec![OsString::from("0.1")],
            working_dir: None,
            env_vars: None,
            expected_max_ram: MemoryAmount::KiloBytes(1)
        }
    }
}