//! [`HeadersFilter`] definition
use crate::extensions::DeserializeExt;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::collections::HashSet;

const SANITIZE_PTR: &str = "/sanitize";
const INCLUDE_PTR: &str = "/include";
const EXCLUDE_PTR: &str = "/exclude";

/// This is uninitialised unless you call [`crate::bootstrap`] or [`crate::logging::init_tracing`] functions.
/// If initialised but env variables are not set fregate will include all headers.///
/// Expects string values separated with ',' or standalone "*" character meaning: all.
/// Example:
/// ```no_run
/// std::env::set_var("TEST_HEADERS_SANITIZE", "password,login,client_id");
/// std::env::set_var("TEST_HEADERS_EXCLUDE", "authorization");
/// std::env::set_var("TEST_HEADERS_INCLUDE", "*");
/// ```
/// In [`crate::extensions::HeaderFilterExt`] trait implementation will have next behaviour:
/// Include all headers except for "authorization" and sanitize "password" header.
pub static HEADER_FILTER: OnceCell<HeadersFilter> = OnceCell::new();

/// Headers filter options
#[derive(Debug, Clone)]
pub enum Filter {
    /// All headers
    All,
    /// Set of header names to filter
    Set(HashSet<String>),
}

/// Struct to save headers filters.
#[derive(Debug, Clone)]
pub struct HeadersFilter {
    /// Headers to be included.
    pub include: Filter,
    /// Headers to be excluded.
    pub exclude: Filter,
    /// Headers to be sanitized.
    pub sanitize: Filter,
}

impl<'de> Deserialize<'de> for HeadersFilter {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let config = Value::deserialize(deserializer)?;

        let sanitize: Option<String> = config
            .pointer_and_deserialize::<_, D::Error>(SANITIZE_PTR)
            .ok();
        let include: Option<String> = config
            .pointer_and_deserialize::<_, D::Error>(INCLUDE_PTR)
            .ok();
        let exclude: Option<String> = config
            .pointer_and_deserialize::<_, D::Error>(EXCLUDE_PTR)
            .ok();

        Ok(HeadersFilter {
            include: from_str_to_filter(include),
            exclude: from_str_to_filter(exclude),
            sanitize: from_str_to_filter(sanitize),
        })
    }
}

fn from_str_to_filter(str: Option<String>) -> Filter {
    str.map(|str| {
        let str = str.trim();

        if str == "*" {
            Filter::All
        } else {
            Filter::Set(
                str.split(',')
                    .map(|field| field.trim().to_ascii_lowercase())
                    .collect::<HashSet<String>>(),
            )
        }
    })
    .unwrap_or(Filter::Set(HashSet::default()))
}
