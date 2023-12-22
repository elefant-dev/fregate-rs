use metrics::{describe_gauge, gauge, register_gauge};
use std::time::Duration;
use sysinfo::{Pid, ProcessExt, System, SystemExt};

pub(crate) fn init_sys_metrics(metrics_update_ms: Duration) {
    let pid = Pid::from(std::process::id() as usize);

    tokio::task::spawn(async move {
        let mut system = System::new();

        loop {
            {
                system.refresh_memory();
                system.refresh_process(pid);
                let process = system.process(pid);

                if let Some(process) = process {
                    gauge!("memory_used", process.memory() as f64);
                    gauge!("cpu_used", process.cpu_usage() as f64);
                }

                gauge!("num_cpus", num_cpus::get() as f64);
                gauge!("memory_available", system.total_memory() as f64);
            }

            tokio::time::sleep(metrics_update_ms).await;
        }
    });
}

pub(crate) fn register_sys_metrics() {
    for (name, describe) in [
        (
            "num_cpus",
            "Returns the number of available CPUs of the current system.",
        ),
        (
            "memory_available",
            "Returns the size of available memory in bytes.",
        ),
        ("memory_used", "Returns the memory usage in bytes."),
        ("cpu_used", "Returns the total CPU usage (in %). Notice that it might be bigger than 100 if run on a multi-core machine."),
    ] {
        describe_gauge!(name, describe);
        register_gauge!(name);
    }
}
