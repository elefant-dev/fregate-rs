use metrics::{describe_gauge, gauge, register_gauge};
use std::convert::Infallible;
use std::io::Read;
use std::num::ParseIntError;
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MaxValue {
    Max,
    Value(u64),
}

impl FromStr for MaxValue {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.trim().eq_ignore_ascii_case("max") {
            Ok(MaxValue::Max)
        } else {
            Ok(MaxValue::Value(u64::from_str(s)?))
        }
    }
}

/// cgroup cpu.max file
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CpuMax {
    max: Option<MaxValue>,
    period: Option<u64>,
}

impl FromStr for CpuMax {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        match s.trim().split_once(' ') {
            Some((max, period)) => {
                let period = u64::from_str(period.trim()).ok();
                let max = MaxValue::from_str(max).ok();

                Ok(CpuMax { max, period })
            }
            None => Ok(CpuMax {
                max: MaxValue::from_str(s).ok(),
                period: None,
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// cgroup cpu.stat file
struct CpuStat {
    usage_usec: Option<u64>,
    user_usec: Option<u64>,
    system_usec: Option<u64>,
    nr_periods: Option<u64>,
    nr_throttled: Option<u64>,
    throttled_usec: Option<u64>,
}

impl FromStr for CpuStat {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.lines();

        let parse_line = |line: &str| match line.split_once(' ') {
            None => None,
            Some((_name, value)) => u64::from_str(value).ok(),
        };

        let usage_usec = iter.next().and_then(parse_line);
        let user_usec = iter.next().and_then(parse_line);
        let system_usec = iter.next().and_then(parse_line);
        let nr_periods = iter.next().and_then(parse_line);
        let nr_throttled = iter.next().and_then(parse_line);
        let throttled_usec = iter.next().and_then(parse_line);

        Ok(Self {
            usage_usec,
            user_usec,
            system_usec,
            nr_periods,
            nr_throttled,
            throttled_usec,
        })
    }
}

fn read_into<T>(file_path: impl AsRef<Path>) -> Option<T>
where
    T: FromStr,
{
    let mut str = String::with_capacity(64);
    let mut file = std::fs::File::open(file_path).ok()?;
    file.read_to_string(&mut str).ok()?;
    T::from_str(str.trim()).ok()
}

fn get_cgroup_path() -> Result<String, Box<dyn std::error::Error>> {
    let mount_point = std::fs::read_to_string("/proc/self/cgroup")?;

    let mut mount = String::new();

    for l in mount_point.lines() {
        let l = l.trim();

        if !l.is_empty() {
            mount = l.to_string();
        } else {
            return Err("Legacy cgroup is not supported.".into());
        }
    }

    match mount.split_once("0::/") {
        None => Err("Could not parse `/proc/self/cgroup` file".into()),
        Some((_, path)) => match path.split_once("(deleted)") {
            None => Ok(format!("/sys/fs/cgroup/{}", path)),
            Some(_) => Err(format!("Cgroup is marked as deleted: `{path}`").into()),
        },
    }
}

pub(crate) fn init_cgroup_metrics(metrics_update_ms: Duration) {
    let cgroup_path = match get_cgroup_path() {
        Ok(path) => path,
        Err(err) => {
            tracing::error!("Could not init cgroup metrics: {err}");
            return;
        }
    };

    let memory_current_path = format!("{cgroup_path}/memory.current");
    let cgroup_mem_available = format!("{cgroup_path}/memory.max");
    let cpu_weight = format!("{cgroup_path}/cpu.weight");
    let cpu_max = format!("{cgroup_path}/cpu.max");
    let cpu_stat = format!("{cgroup_path}/cpu.stat");

    tokio::task::spawn(async move {
        loop {
            let num_cpu = num_cpus::get() as u64;
            let cgroup_mem_used = read_into::<u64>(memory_current_path.as_str());
            let cgroup_mem_available = read_into::<MaxValue>(cgroup_mem_available.as_str());
            let cpu_weight = read_into::<u64>(cpu_weight.as_str());
            let cpu_max = read_into::<CpuMax>(cpu_max.as_str());
            let cpu_stat = read_into::<CpuStat>(cpu_stat.as_str());

            if let Some(CpuStat {
                usage_usec,
                user_usec,
                system_usec,
                nr_periods,
                nr_throttled,
                throttled_usec,
            }) = cpu_stat
            {
                set_gauge_from_option("cpu.stat.usage_usec", usage_usec);
                set_gauge_from_option("cpu.stat.user_usec", user_usec);
                set_gauge_from_option("cpu.stat.system_usec", system_usec);
                set_gauge_from_option("cpu.stat.nr_periods", nr_periods);
                set_gauge_from_option("cpu.stat.nr_throttled", nr_throttled);
                set_gauge_from_option("cpu.stat.throttled_usec", throttled_usec);
            }
            if let Some(CpuMax { max, period }) = cpu_max {
                set_gauge_from_max_option("cpu.max.max", max);
                set_gauge_from_option("cpu.max.period", period);
            }
            set_gauge_from_option("cpu.weight", cpu_weight);
            set_gauge_from_option("num_cpus", Some(num_cpu));
            set_gauge_from_option("memory_used", cgroup_mem_used);
            set_gauge_from_max_option("memory_available", cgroup_mem_available);

            tokio::time::sleep(metrics_update_ms).await;
        }
    });
}

fn set_gauge_from_option(name: &'static str, v: Option<u64>) {
    if let Some(v) = v.map(|v| v as f64) {
        gauge!(name, v)
    };
}

fn set_gauge_from_max_option(name: &'static str, v: Option<MaxValue>) {
    match v {
        Some(MaxValue::Value(v)) => gauge!(name, v as f64),
        Some(MaxValue::Max) => gauge!(name, -1_f64),
        _ => {}
    };
}

pub(crate) fn register_cgroup_metrics() {
    for (name, describe) in [
        (
            "num_cpus",
            "Returns the number of available CPUs of the current system.",
        ),
        (
            "memory_available",
            "Returns the size of available memory in bytes. `-1` means no limit.",
        ),
        ("memory_used", "Returns the memory usage in bytes."),
        (
            "cpu.stat.usage_usec",
            "Returns total cpu time for group in us",
        ),
        (
            "cpu.stat.user_usec",
            "Returns user cpu time for group in us",
        ),
        (
            "cpu.stat.system_usec",
            "Returns system cpu time for group in us",
        ),
        (
            "cpu.stat.nr_periods",
            "Returns how many full periods have been elapsed",
        ),
        (
            "cpu.stat.nr_throttled",
            "Returns number of times we exausted the full allowed bandwidth",
        ),
        (
            "cpu.stat.throttled_usec",
            "Returns total time the tasks were not run due to being overquota",
        ),
        (
            "cpu.max.max",
            "Returns maximum bandwidth limit. `-1` means no limit.",
        ),
        (
            "cpu.max.period",
            "Returns the period in which group may consume `cpu.max.max` of cpu bandwidth.",
        ),
        ("cpu.weight", "."),
    ] {
        describe_gauge!(name, describe);
        register_gauge!(name);
    }
}
