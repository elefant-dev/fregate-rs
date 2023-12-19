use metrics::{describe_gauge, gauge, register_gauge};
use std::sync::{Mutex, OnceLock, PoisonError};
use std::time::Duration;
use sysinfo::{CpuExt, Pid, ProcessExt, System, SystemExt};

/// It is recommended to keep only 1 instance of System
pub static SYSTEM: OnceLock<Mutex<System>> = OnceLock::new();

pub fn init_sys_metrics(metrics_update_ms: Duration) {
    let pid = Pid::from(std::process::id() as usize);

    tokio::task::spawn(async move {
        loop {
            {
                let sys = SYSTEM.get_or_init(|| Mutex::new(System::new()));
                let mut sys = sys.lock().unwrap_or_else(PoisonError::into_inner);

                sys.refresh_process(pid);
                let process = sys.process(pid);

                if let Some(process) = process {
                    gauge!("current_process_memory_usage", process.memory() as f64);
                    gauge!("current_process_cpu_usage", process.cpu_usage() as f64);
                }

                let cpu_info = sys.global_cpu_info();

                gauge!("system_memory_usage", sys.used_memory() as f64);
                gauge!("system_cpu_usage", cpu_info.cpu_usage() as f64);
                gauge!("num_cpus", num_cpus::get() as f64);
                gauge!("total_available_memory", sys.total_memory() as f64);
            }

            tokio::time::sleep(metrics_update_ms).await;
        }
    });
}

pub fn register_sys_metrics() {
    for (name, describe) in [
        (
            "num_cpus",
            "Returns the number of available CPUs of the current system.",
        ),
        (
            "total_available_memory",
            "Returns the size of available memory in bytes.",
        ),
        (
            "current_process_memory_usage",
            "Returns the memory usage in bytes.",
        ),
        (
            "current_process_cpu_usage",
            "Returns the total CPU usage in %.",
        ),
        ("system_memory_usage", "Returns the memory usage in bytes."),
        ("system_cpu_usage", "Returns the total CPU usage in %."),
    ] {
        describe_gauge!(name, describe);
        register_gauge!(name);
    }
}
