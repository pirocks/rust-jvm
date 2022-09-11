#![feature(never_type)]

pub struct PerfMetrics {
    enabled: bool,
    // exit_times: ResponseTime<AtomicHdrHistogram,StdInstantMicros>,
    // checkcast_times: ResponseTime<AtomicHdrHistogram,StdInstantMicros>,
    // throw_times: ResponseTime<AtomicHdrHistogram,StdInstantMicros>,
    // allocate_object_times: ResponseTime<AtomicHdrHistogram,StdInstantMicros>,
    // get_static_times: ResponseTime<AtomicHdrHistogram,StdInstantMicros>,
    // compilations: ResponseTime<AtomicHdrHistogram,StdInstantMicros>,
    // verifier: ResponseTime<AtomicHdrHistogram,StdInstantMicros>
}

impl PerfMetrics {
    pub fn new() -> Self {
        Self {
            enabled: false,
            // exit_times: ResponseTime::default(),
            // checkcast_times: Default::default(),
            // throw_times: Default::default(),
            // allocate_object_times: Default::default(),
            // get_static_times: Default::default(),
            // compilations: ResponseTime::default(),
            // verifier: Default::default()
        }
    }

    pub fn vm_exit_start(&self) -> VMExitGuard {
        if self.enabled {
            // let enter_instant = self.exit_times.enter();
            // VMExitGuard::Enabled {
            //     enter_instant,
            //     metrics: self
            // }
            todo!()
        } else {
            VMExitGuard::Disabled
        }
    }

    pub fn vm_exit_get_static(&self) -> GetStaticGuardType {
        if self.enabled {
            // let enter_instant = self.get_static_times.enter();
            // GetStaticGuardType::Enabled {
            //     enter_instant,
            //     metrics: self
            // }
            todo!()
        } else {
            GetStaticGuardType::Disabled
        }
    }

    pub fn vm_exit_checkcast(&self) -> CheckCastGuardType {
        if self.enabled {
            // let enter_instant = self.checkcast_times.enter();
            // CheckCastGuardType::Enabled {
            //     enter_instant,
            //     metrics: self
            // }
            todo!()
        } else {
            CheckCastGuardType::Disabled
        }
    }

    pub fn vm_exit_throw(&self) -> ThrowGuardType {
        if self.enabled {
            // let enter_instant = self.throw_times.enter();
            // ThrowGuardType::Enabled {
            //     enter_instant,
            //     metrics: self
            // }
            todo!()
        } else {
            ThrowGuardType::Disabled
        }
    }

    pub fn vm_exit_allocate_obj(&self) -> AllocateObjectGuardType {
        if self.enabled {
            // let enter_instant = self.allocate_object_times.enter();
            // AllocateObjectGuardType::Enabled {
            //     enter_instant,
            //     metrics: self
            // }
            todo!()
        } else {
            AllocateObjectGuardType::Disabled
        }
    }

    pub fn compilation_start(&self) -> CompilationGuard {
        if self.enabled {
            // let enter_instant = self.compilations.enter();
            // CompilationGuard::Enabled {
            //     enter_instant,
            //     metrics: self
            // }
            todo!()
        } else {
            CompilationGuard::Disabled
        }
    }

    pub fn verifier_start(&self) -> VerifyGuard {
        if self.enabled {
            // let enter_instant = self.verifier.enter();
            // VerifyGuard::Enabled {
            //     enter_instant,
            //     metrics: self
            // }
            todo!()
        } else {
            VerifyGuard::Disabled
        }
    }


    pub fn display(&self) {
        // let exit_histogram = self.exit_times.histogram();
        // println!("Exits:\n{}", serde_yaml::to_string(&exit_histogram).unwrap());
        // let compilation_histogram = self.compilations.histogram();
        // println!("Compilation:\n{}", serde_yaml::to_string(&compilation_histogram).unwrap());
        // let checkcast_histogram = self.checkcast_times.histogram();
        // println!("Checkcast:\n{}", serde_yaml::to_string(&checkcast_histogram).unwrap());
        // let throw_histogram = self.throw_times.histogram();
        // println!("Throw:\n{}", serde_yaml::to_string(&throw_histogram).unwrap());
        // let allocate_object_histogram = self.allocate_object_times.histogram();
        // println!("Allocate Object:\n{}", serde_yaml::to_string(&allocate_object_histogram).unwrap());
        // let get_static_histogram = self.get_static_times.histogram();
        // println!("Get Static:\n{}", serde_yaml::to_string(&get_static_histogram).unwrap());
        // let verifier_histogram = self.verifier.histogram();
        // println!("Verify:\n{}", serde_yaml::to_string(&verifier_histogram).unwrap());
        todo!()
    }
}

pub enum CompilationGuard<'l> {
    Enabled {
        enter_instant: !/*StdInstantMicros*/,
        metrics: &'l PerfMetrics,
    },
    Disabled,
}

impl Drop for CompilationGuard<'_> {
    fn drop(&mut self) {
        match self {
            CompilationGuard::Enabled { ../*enter_instant, metrics*/ } => {
                todo!()
                // let instant = enter_instant.clone();
                // OnResult::<()>::leave_scope(&metrics.compilations, instant);
            }
            CompilationGuard::Disabled => {}
        }
    }
}

pub enum VerifyGuard<'l> {
    Enabled {
        enter_instant: !/*StdInstantMicros*/,
        metrics: &'l PerfMetrics,
    },
    Disabled,
}

impl Drop for VerifyGuard<'_> {
    fn drop(&mut self) {
        match self {
            VerifyGuard::Enabled { ../*enter_instant, metrics*/ } => {
                // let instant = enter_instant.clone();
                // OnResult::<()>::leave_scope(&metrics.verifier, instant);
                todo!()
            }
            VerifyGuard::Disabled => {}
        }
    }
}

pub enum VMExitGuard<'l> {
    Enabled {
        enter_instant: !/*StdInstantMicros*/,
        metrics: &'l PerfMetrics,
    },
    Disabled,
}

impl Drop for VMExitGuard<'_> {
    fn drop(&mut self) {
        match self {
            VMExitGuard::Enabled { ../*enter_instant, metrics*/ } => {
                // let instant = enter_instant.clone();
                // OnResult::<()>::leave_scope(&metrics.exit_times, instant);
                todo!()
            }
            VMExitGuard::Disabled => {}
        }
    }
}

pub enum CheckCastGuardType<'l> {
    Enabled {
        enter_instant: !/*StdInstantMicros*/,
        metrics: &'l PerfMetrics,
    },
    Disabled,
}

impl Drop for CheckCastGuardType<'_> {
    fn drop(&mut self) {
        match self {
            CheckCastGuardType::Enabled { ../*enter_instant, metrics*/ } => {
                // let instant = enter_instant.clone();
                // OnResult::<()>::leave_scope(&metrics.checkcast_times, instant);
                todo!()
            }
            CheckCastGuardType::Disabled => {}
        }
    }
}


pub enum GetStaticGuardType<'l> {
    Enabled {
        enter_instant: !/*StdInstantMicros*/,
        metrics: &'l PerfMetrics,
    },
    Disabled,
}

impl Drop for GetStaticGuardType<'_> {
    fn drop(&mut self) {
        match self {
            GetStaticGuardType::Enabled { ../*enter_instant, metrics*/ } => {
                // let instant = enter_instant.clone();
                // OnResult::<()>::leave_scope(&metrics.get_static_times, instant);
                todo!()
            }
            GetStaticGuardType::Disabled => {}
        }
    }
}


pub enum ThrowGuardType<'l> {
    Enabled {
        enter_instant: !/*StdInstantMicros*/,
        metrics: &'l PerfMetrics,
    },
    Disabled,
}

impl Drop for ThrowGuardType<'_> {
    fn drop(&mut self) {
        match self {
            ThrowGuardType::Enabled { ../*enter_instant, metrics*/ } => {
                // let instant = enter_instant.clone();
                // OnResult::<()>::leave_scope(&metrics.throw_times, instant);
                todo!()
            }
            ThrowGuardType::Disabled => {}
        }
    }
}

pub enum AllocateObjectGuardType<'l> {
    Enabled {
        enter_instant: !/*StdInstantMicros*/,
        metrics: &'l PerfMetrics,
    },
    Disabled,
}

impl Drop for AllocateObjectGuardType<'_> {
    fn drop(&mut self) {
        match self {
            AllocateObjectGuardType::Enabled { ../*enter_instant, metrics*/ } => {
                // let instant = enter_instant.clone();
                // OnResult::<()>::leave_scope(&metrics.allocate_object_times, instant);
                todo!()
            }
            AllocateObjectGuardType::Disabled => {}
        }
    }
}