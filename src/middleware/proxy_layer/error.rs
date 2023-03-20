//!Errors [`crate::middleware::proxy_layer::ProxyLayer`] may return in runtime.

use hyper::http;
use tower_http::BoxError;

#[derive(thiserror::Error, Debug)]
///Errors enum.
pub enum ProxyError {
    #[error("`{0}`")]
    /// Returned if fail to build new uri for [`hyper::Request`]
    UriBuilder(http::Error),
    #[error("`{0}`")]
    /// Returned on any other error while sending [`hyper::Request`]
    SendRequest(BoxError),
}

/// Result Alias
pub type ProxyResult<T> = Result<T, ProxyError>;
