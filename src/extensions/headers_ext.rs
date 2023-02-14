use crate::observability::{Filter, HeadersFilter, HEADERS_FILTER, SANITIZED_VALUE};
use axum::headers::{HeaderMap, HeaderName};
use hyper::http::HeaderValue;
use std::borrow::Cow;

/// Extension trait to get filtered headers.
/// Current implementation relies on [`HEADERS_FILTER`].
#[sealed::sealed]
pub trait HeaderFilterExt
where
    Self: Clone,
{
    /// Extension trait to get filtered values.
    fn get_filtered(&self) -> Cow<'_, Self>;
}

#[sealed::sealed]
impl HeaderFilterExt for HeaderMap {
    /// If [`HEADERS_FILTER`] is uninitialised returns [`Cow::Borrowed`] otherwise creates clone and returns [`Cow::Owned`] from included and sanitized fields.
    fn get_filtered(&self) -> Cow<'_, Self> {
        HEADERS_FILTER
            .get()
            .map(
                |HeadersFilter {
                     sanitize,
                     exclude,
                     include,
                 }| {
                    let filtered = self
                        .iter()
                        .map(|(name, value)| {
                            let lowercase = name.as_str().to_ascii_lowercase();
                            (lowercase, name, value)
                        })
                        .filter_map(|(lowercase, name, value)| {
                            include_value(include, lowercase, name, value)
                        })
                        .filter_map(|(lowercase, name, value)| {
                            exclude_value(exclude, lowercase, name, value)
                        })
                        .map(|(lowercase, name, value)| {
                            sanitize_value(sanitize, lowercase, name, value)
                        })
                        .collect();
                    Cow::Owned(filtered)
                },
            )
            .unwrap_or(Cow::Borrowed(self))
    }
}

fn sanitize_value<'a>(
    sanitize: &'a Filter,
    lowercase: String,
    name: &'a HeaderName,
    value: &'a HeaderValue,
) -> (HeaderName, HeaderValue) {
    match sanitize {
        Filter::All => (name.clone(), HeaderValue::from_static(SANITIZED_VALUE)),
        Filter::Set(set) => {
            if set.contains(&lowercase) {
                (name.clone(), HeaderValue::from_static(SANITIZED_VALUE))
            } else {
                (name.clone(), value.clone())
            }
        }
    }
}

fn include_value<'a>(
    include: &'a Filter,
    lowercase: String,
    name: &'a HeaderName,
    value: &'a HeaderValue,
) -> Option<(String, &'a HeaderName, &'a HeaderValue)> {
    match include {
        Filter::All => Some((lowercase, name, value)),
        Filter::Set(set) => {
            if set.contains(&lowercase) {
                Some((lowercase, name, value))
            } else {
                None
            }
        }
    }
}

fn exclude_value<'a>(
    exclude: &'a Filter,
    lowercase: String,
    name: &'a HeaderName,
    value: &'a HeaderValue,
) -> Option<(String, &'a HeaderName, &'a HeaderValue)> {
    match exclude {
        Filter::All => None,
        Filter::Set(set) => {
            if set.contains(&lowercase) {
                None
            } else {
                Some((lowercase, name, value))
            }
        }
    }
}
