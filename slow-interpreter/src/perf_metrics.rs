use metered::{Enter, ResponseTime};
use metered::hdr_histogram::AtomicHdrHistogram;
use metered::metric::OnResult;
use metered::time_source::{StdInstant, StdInstantMicros};

pub struct PerfMetrics {
    enabled: bool,
    exit_times: ResponseTime<AtomicHdrHistogram,StdInstantMicros>,
    compilations: ResponseTime<AtomicHdrHistogram,StdInstantMicros>
}

impl PerfMetrics {
    pub fn new() -> Self {
        Self {
            enabled: true,
            exit_times: ResponseTime::default(),
            compilations: ResponseTime::default()
        }
    }

    pub fn vm_exit_start(&self) -> VMExitGuard{
        if self.enabled{
            let enter_instant = self.exit_times.enter();
            VMExitGuard::Enabled {
                enter_instant,
                metrics: self
            }
        }else {
            VMExitGuard::Disabled
        }
    }

    pub fn compilation_start(&self) -> CompilationGuard{
        if self.enabled{
            let enter_instant = self.compilations.enter();
            CompilationGuard::Enabled {
                enter_instant,
                metrics: self
            }
        }else {
            CompilationGuard::Disabled
        }
    }

    pub fn display(&self) {
        let exit_histogram = self.exit_times.histogram();
        println!("{}", serde_yaml::to_string(&exit_histogram).unwrap());
        let compilation_histogram = self.compilations.histogram();
        println!("{}", serde_yaml::to_string(&compilation_histogram).unwrap());
    }
}

pub enum CompilationGuard<'l>{
    Enabled{
        enter_instant: StdInstantMicros,
        metrics: &'l PerfMetrics
    },
    Disabled
}

impl Drop for CompilationGuard<'_> {
    fn drop(&mut self) {
        match self {
            CompilationGuard::Enabled { enter_instant, metrics } => {
                let instant = enter_instant.clone();
                OnResult::<()>::leave_scope(&metrics.compilations, instant);
            }
            CompilationGuard::Disabled => {}
        }
    }
}

pub enum  VMExitGuard<'l>{
    Enabled{
        enter_instant: StdInstantMicros,
        metrics: &'l PerfMetrics
    },
    Disabled
}

impl Drop for VMExitGuard<'_> {
    fn drop(&mut self) {
        match self {
            VMExitGuard::Enabled { enter_instant, metrics } => {
                let instant = enter_instant.clone();
                OnResult::<()>::leave_scope(&metrics.exit_times, instant);
            }
            VMExitGuard::Disabled => {}
        }
    }
}