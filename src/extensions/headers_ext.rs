use crate::headers::{Filter, HeadersFilter, HEADERS_FILTER};
use crate::logging::SANITIZED_VALUE;
use axum::headers::HeaderMap;
use hyper::http::HeaderValue;
use std::borrow::Cow;

/// Extension trait to get filtered headers.
/// Current implementation relies on [`HEADERS_FILTER`].
pub trait HeaderFilterExt
where
    Self: Clone,
{
    /// Extension trait to get filtered values.
    fn get_filtered(&self) -> Cow<'_, Self>;
}

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
                        .filter_map(|(lowercase, name, value)| match include {
                            Filter::All => Some((lowercase, name, value)),
                            Filter::Set(set) => {
                                if set.contains(&lowercase) {
                                    Some((lowercase, name, value))
                                } else {
                                    None
                                }
                            }
                        })
                        .filter_map(|(lowercase, name, value)| match exclude {
                            Filter::All => None,
                            Filter::Set(set) => {
                                if set.contains(&lowercase) {
                                    None
                                } else {
                                    Some((lowercase, name, value))
                                }
                            }
                        })
                        .map(|(lowercase, name, value)| match sanitize {
                            Filter::All => {
                                (name.clone(), HeaderValue::from_static(SANITIZED_VALUE))
                            }
                            Filter::Set(set) => {
                                if set.contains(&lowercase) {
                                    (name.clone(), HeaderValue::from_static(SANITIZED_VALUE))
                                } else {
                                    (name.clone(), value.clone())
                                }
                            }
                        })
                        .collect();
                    Cow::Owned(filtered)
                },
            )
            .unwrap_or(Cow::Borrowed(self))
    }
}
