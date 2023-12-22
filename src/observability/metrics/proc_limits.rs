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

pub(crate) fn read_process_limits(pid: u32) -> Result<Limits, Box<dyn Error>> {
    let limits = std::fs::read_to_string(format!("/proc/{pid}/limits"))?;
    read_limits(limits)
}

fn read_limits(limits: String) -> Result<Limits, Box<dyn Error>> {
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

    Err(format!("Could not find `Max cpu time` soft limit").into())
}

#[cfg(test)]
mod read_test {
    use super::read_limits;
    use super::UnlimitedValue;

    #[test]
    fn read_max_cpu_time_unlimited() {
        let limits =
            r#"Limit                     Soft Limit           Hard Limit           Units     
Max cpu time              unlimited            unlimited            seconds   
Max file size             unlimited            unlimited            bytes     
Max data size             unlimited            unlimited            bytes     
Max stack size            8388608              unlimited            bytes     
Max core file size        0                    unlimited            bytes     
Max resident set          unlimited            unlimited            bytes     
Max processes             unlimited            unlimited            processes 
Max open files            1048576              1048576              files     
Max locked memory         65536                65536                bytes     
Max address space         unlimited            unlimited            bytes     
Max file locks            unlimited            unlimited            locks     
Max pending signals       63487                63487                signals   
Max msgqueue size         819200               819200               bytes     
Max nice priority         0                    0                    
Max realtime priority     0                    0                    
Max realtime timeout      unlimited            unlimited            us"#
                .to_owned();

        let limits = read_limits(limits).unwrap();
        assert_eq!(
            limits.max_cpu_limit.soft_limit,
            Some(UnlimitedValue::Unlimited)
        )
    }

    #[test]
    fn read_max_cpu_time_exact() {
        let limits =
            r#"Limit                     Soft Limit           Hard Limit           Units     
Max cpu time              8388608            unlimited            seconds   
Max file size             unlimited            unlimited            bytes     
Max data size             unlimited            unlimited            bytes     
Max stack size            8388608              unlimited            bytes     
Max core file size        0                    unlimited            bytes     
Max resident set          unlimited            unlimited            bytes     
Max processes             unlimited            unlimited            processes 
Max open files            1048576              1048576              files     
Max locked memory         65536                65536                bytes     
Max address space         unlimited            unlimited            bytes     
Max file locks            unlimited            unlimited            locks     
Max pending signals       63487                63487                signals   
Max msgqueue size         819200               819200               bytes     
Max nice priority         0                    0                    
Max realtime priority     0                    0                    
Max realtime timeout      unlimited            unlimited            us"#
                .to_owned();

        let limits = read_limits(limits).unwrap();
        assert_eq!(
            limits.max_cpu_limit.soft_limit,
            Some(UnlimitedValue::Value(8388608))
        )
    }
}
