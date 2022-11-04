use crate::AppConfig;
use metrics::{
    absolute_counter, describe_counter, describe_gauge, describe_histogram, gauge, histogram,
    register_counter, register_gauge, register_histogram,
};
use tokio::runtime::Handle;
use tokio_metrics::{RuntimeMetrics, RuntimeMonitor};

pub(crate) fn init_tokio_metrics_task<T>(config: &AppConfig<T>) {
    let handle = Handle::current();
    let runtime_monitor = RuntimeMonitor::new(&handle);

    let metrics_update_interval = config.metrics_update_interval;
    tokio::task::spawn(async move {
        for RuntimeMetrics {
            workers_count,
            total_park_count,
            max_park_count,
            min_park_count,
            total_noop_count,
            max_noop_count,
            min_noop_count,
            total_steal_count,
            max_steal_count,
            min_steal_count,
            num_remote_schedules,
            total_local_schedule_count,
            max_local_schedule_count,
            min_local_schedule_count,
            total_overflow_count,
            max_overflow_count,
            min_overflow_count,
            total_polls_count,
            max_polls_count,
            min_polls_count,
            total_busy_duration,
            max_busy_duration,
            min_busy_duration,
            injection_queue_depth,   // inc / dec
            total_local_queue_depth, // inc / dec
            max_local_queue_depth,
            min_local_queue_depth,
            elapsed,
            ..
        } in runtime_monitor.intervals()
        {
            absolute_counter!(
                "workers_count",
                workers_count.try_into().unwrap_or(u64::MAX)
            );
            absolute_counter!("total_park_count", total_park_count);
            absolute_counter!("max_park_count", max_park_count);
            absolute_counter!("min_park_count", min_park_count);
            absolute_counter!("total_noop_count", total_noop_count);
            absolute_counter!("max_noop_count", max_noop_count);
            absolute_counter!("min_noop_count", min_noop_count);
            absolute_counter!("total_steal_count", total_steal_count);
            absolute_counter!("max_steal_count", max_steal_count);
            absolute_counter!("min_steal_count", min_steal_count);
            absolute_counter!("num_remote_schedules", num_remote_schedules);
            absolute_counter!("total_local_schedule_count", total_local_schedule_count);
            absolute_counter!("max_local_schedule_count", max_local_schedule_count);
            absolute_counter!("min_local_schedule_count", min_local_schedule_count);
            absolute_counter!("total_overflow_count", total_overflow_count);
            absolute_counter!("max_overflow_count", max_overflow_count);
            absolute_counter!("min_overflow_count", min_overflow_count);
            absolute_counter!("total_polls_count", total_polls_count);
            absolute_counter!("max_polls_count", max_polls_count);
            absolute_counter!("min_polls_count", min_polls_count);
            histogram!("total_busy_duration", total_busy_duration);
            histogram!("max_busy_duration", max_busy_duration);
            histogram!("min_busy_duration", min_busy_duration);
            gauge!(
                "injection_queue_depth",
                usize_to_f64_saturated(injection_queue_depth)
            );
            gauge!(
                "total_local_queue_depth",
                usize_to_f64_saturated(total_local_queue_depth)
            );
            absolute_counter!(
                "max_local_queue_depth",
                max_local_queue_depth.try_into().unwrap_or(u64::MAX)
            );
            absolute_counter!(
                "min_local_queue_depth",
                min_local_queue_depth.try_into().unwrap_or(u64::MAX)
            );
            histogram!("elapsed", elapsed);
            tokio::time::sleep(metrics_update_interval).await;
        }
    });
}

pub(crate) fn register_metrics() {
    for (name, describe) in [
        (
            "workers_count",
            "The number of worker threads used by the runtime.",
        ),
        (
            "total_park_count",
            "The number of times worker threads parked.",
        ),
        ("max_park_count", "The maximum number of times any worker thread parked."),
        ("min_park_count", "The minimum number of times any worker thread parked."),
        ("total_noop_count", "The number of times worker threads unparked but performed no work before parking again."),
        ("max_noop_count", "The maximum number of times any worker thread unparked but performed no work before parking again."),
        ("min_noop_count", "The minimum number of times any worker thread unparked but performed no work before parking again."),
        ("total_steal_count", "The number of times worker threads stole tasks from another worker thread."),
        ("max_steal_count", "The maximum number of times any worker thread stole tasks from another worker thread."),
        ("min_steal_count", "The minimum number of times any worker thread stole tasks from another worker thread."),
        ("num_remote_schedules", "The number of tasks scheduled from outside of the runtime."),
        ("total_local_schedule_count", "The number of tasks scheduled from worker threads."),
        ("max_local_schedule_count", "The maximum number of tasks scheduled from any one worker thread."),
        ("min_local_schedule_count", "The minimum number of tasks scheduled from any one worker thread."),
        ("total_overflow_count", "The number of times worker threads saturated their local queues."),
        ("max_overflow_count", "The maximum number of times any one worker saturated its local queue."),
        ("min_overflow_count", "The minimum number of times any one worker saturated its local queue."),
        ("total_polls_count", "The number of tasks that have been polled across all worker threads."),
        ("max_polls_count", "The maximum number of tasks that have been polled in any worker thread."),
        ("min_polls_count", "The minimum number of tasks that have been polled in any worker thread."),
        ("max_local_queue_depth", "The maximum number of tasks currently scheduled any worker's local queue."),
        ("min_local_queue_depth", "The minimum number of tasks currently scheduled any worker's local queue."),
    ] {
        describe_counter!(name, describe);
        register_counter!(name);
    }

    for (name, describe) in [
        (
            "total_busy_duration",
            "The amount of time (in seconds) worker threads were busy.",
        ),
        (
            "max_busy_duration",
            "The maximum amount of time (in seconds) a worker thread was busy.",
        ),
        (
            "min_busy_duration",
            "The minimum amount of time (in seconds) a worker thread was busy.",
        ),
        (
            "elapsed",
            "Total amount of time (in seconds) elapsed since observing runtime metrics.",
        ),
    ] {
        describe_histogram!(name, describe);
        register_histogram!(name);
    }

    for (name, describe) in [
        (
            "injection_queue_depth",
            "The number of tasks currently scheduled in the runtime's injection queue.",
        ),
        (
            "total_local_queue_depth",
            "The total number of tasks currently scheduled in workers' local queues.",
        ),
    ] {
        describe_gauge!(name, describe);
        register_gauge!(name);
    }
}

const fn usize_to_f64_saturated(n: usize) -> f64 {
    let ret = n as f64;

    if ret as usize != n {
        f64::MAX
    } else {
        ret
    }
}
