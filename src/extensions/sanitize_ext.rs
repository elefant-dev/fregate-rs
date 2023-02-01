use crate::logging::{SANITIZED_VALUE, SANITIZE_FIELDS};
use axum::headers::HeaderMap;
use hyper::http::HeaderValue;
use std::borrow::Cow;

/// Extension trait to get sanitized values.
pub trait SanitizeExt
where
    Self: Clone,
{
    /// Extension trait to get sanitized values.
    fn get_sanitized(&self) -> Cow<'_, Self>;
}

impl SanitizeExt for HeaderMap {
    /// Uses [`SANITIZE_FIELDS`] to find keys to be sanitized.
    /// If [`SANITIZE_FIELDS`] is uninitialised return [`Cow::Borrowed`] otherwise creates clone and returns [`Cow::Owned`] with [`SANITIZED_VALUE`].
    fn get_sanitized(&self) -> Cow<'_, Self> {
        SANITIZE_FIELDS
            .get()
            .map(|sanitize_fields| {
                let mut sanitized_map = HeaderMap::with_capacity(self.len());

                self.iter().for_each(|(name, value)| {
                    let lowercase = name.as_str().to_ascii_lowercase();

                    if sanitize_fields.contains(&lowercase) {
                        sanitized_map.insert(name, HeaderValue::from_static(SANITIZED_VALUE));
                    } else {
                        sanitized_map.insert(name, value.clone());
                    }
                });

                Cow::Owned(sanitized_map)
            })
            .unwrap_or(Cow::Borrowed(self))
    }
}
