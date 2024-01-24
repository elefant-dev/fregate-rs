use metrics::{describe_gauge, gauge, register_gauge};
use std::time::Duration;
use sysinfo::{Pid, ProcessExt, System, SystemExt};

pub fn init_sys_metrics(metrics_update_ms: Duration) {
    let u32_pid = std::process::id();
    let pid = Pid::from(u32_pid as usize);

    tokio::task::spawn(async move {
        let mut system = System::new();

        loop {
            {
                #[cfg(target_os = "linux")]
                {
                    use crate::observability::proc_limits::{read_process_limits, UnlimitedValue};

                    match read_process_limits(u32_pid) {
                        Ok(limits) => match limits.max_cpu_limit.soft_limit {
                            Some(UnlimitedValue::Unlimited) => gauge!("max_cpu_time", -1_f64),
                            Some(UnlimitedValue::Value(v)) => gauge!("max_cpu_time", v as f64),
                            _ => {}
                        },
                        Err(err) => {
                            tracing::error!("Could not update limits: {err}");
                        }
                    }
                }

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

    #[cfg(target_os = "linux")]
    {
        describe_gauge!(
            "max_cpu_time",
            "Returns max cpu time soft limit in seconds. `-1` means `unlimited`"
        );
        register_gauge!("max_cpu_time");
    }
}
