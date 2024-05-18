use std::future::Future;
use itertools::Itertools;
use sysinfo::{CpuRefreshKind, RefreshKind, System};
use tokio::{spawn, try_join};
use tokio::task::spawn_blocking;
use tokio::time::Instant;

use pressure_stall_information_parser::PressureStallInformation;

pub struct PressureMaximizingExecutor<T: Send + 'static, F: Future<Output=T> + Send + 'static, I: Iterator<Item=F>> {
    pub system: System,
    pub initial: u64,
    pub cpu_max_pressure: f64,
    pub io_max_pressure: f64,
    pub mem_max_pressure: f64,
    pub to_execute: Option<I>,
    pub start_instant: Instant,
}

impl<T: Send + 'static, F: Future<Output=T> + Send + 'static, I: Iterator<Item=F>> PressureMaximizingExecutor<T, F, I> {
    pub async fn new(to_execute: I) -> Self {
        let system = System::new_with_specifics(
            RefreshKind::new().with_cpu(CpuRefreshKind::everything()),
        );

        let initial = system.cpus().len() as u64;
        let start_instant= Instant::now();
        Self {
            system,
            initial,
            cpu_max_pressure: 5.0,
            io_max_pressure: 400.0,
            start_instant,
            to_execute: Some(to_execute),
            mem_max_pressure: 5.0,
        }
    }

    pub async fn execute(mut self) -> Vec<T> {
        let mut join_handles = vec![];
        for item in self.to_execute.take().unwrap() {
            while !self.should_continue_spawning().await {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
            join_handles.push(spawn(item));
        }
        let mut res = vec![];
        for join_handle in join_handles {
            res.push(join_handle.await.unwrap());
        }
        res
    }

    //todo psi has a notification mechanism, so like shouldn't be polling manually
    async fn should_continue_spawning(&mut self) -> bool {
        let (io_pressure, mem_pressure, cpu_pressure) = self.current_pressures().await;
        self.system.refresh_cpu();
        let cpu_usages = self.system.cpus().iter().map(|cpu|cpu.cpu_usage()).collect_vec();
        let num_cpus = cpu_usages.len() as f32;
        let average: f32 = cpu_usages.into_iter().sum::<f32>()/num_cpus;
        if dbg!(average) > 98.0 {
            return false;
        }
        if Instant::now().checked_duration_since(self.start_instant).unwrap().as_secs() < 10{
            if self.initial == 0{
                return false;
            }
            self.initial -= 1;
            return true;
        }
        dbg!(io_pressure.some.avg10) < self.io_max_pressure && dbg!(mem_pressure.some.avg10) < self.mem_max_pressure && dbg!(cpu_pressure.some.avg10) < self.cpu_max_pressure
    }

    pub async fn current_pressures(&self) -> (PressureStallInformation, PressureStallInformation, PressureStallInformation) {
        let (io_pressure, mem_pressure, cpu_pressure) = try_join!(
            spawn_blocking(||{
                PressureStallInformation::io_pressure()
            }),
            spawn_blocking(||{
                PressureStallInformation::mem_pressure()
            }),
            spawn_blocking(||{
                PressureStallInformation::cpu_pressure()
            })
        ).unwrap();
        (io_pressure.unwrap(), mem_pressure.unwrap(), cpu_pressure.unwrap())
    }
}
