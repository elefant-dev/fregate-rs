use std::error::Error;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub(crate) struct Limits {
    pub(crate) max_cpu_limit: Limit,
}

#[derive(Debug, Clone)]
pub(crate) struct Limit {
    pub(crate) soft_limit: Option<UnlimitedValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UnlimitedValue {
    Unlimited,
    Value(u64),
}

pub(crate) fn read_limits(pid: u32) -> Result<Limits, Box<dyn Error>> {
    let limits = std::fs::read_to_string(format!("/proc/{pid}/limits"))?;

    for line in limits.lines() {
        if let Some((_, values)) = line.split_once("Max cpu time") {
            let mut values = values.split_whitespace();

            let soft_limit = values.next().and_then(|soft_limit| match soft_limit {
                "unlimited" => Some(UnlimitedValue::Unlimited),
                _ => u64::from_str(soft_limit.trim())
                    .ok()
                    .map(UnlimitedValue::Value),
            });

            return Ok(Limits {
                max_cpu_limit: Limit { soft_limit },
            });
        }
    }

    Err(format!("Could not find `Max cpu time` soft limit for pid: `{pid}`").into())
}
