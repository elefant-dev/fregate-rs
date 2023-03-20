//! Layer that applies [`ProxyService`] to [`Request`].\
//! See in [examples](https://github.com/elefant-dev/fregate-rs/blob/main/examples/proxy-layer/src/main.rs) how it might be used
use crate::middleware::proxy_layer::error::ProxyError;
use crate::middleware::proxy_layer::service::ProxyService;
use crate::middleware::proxy_layer::shared::Shared;
use axum::body::{Bytes, HttpBody};
use axum::response::Response as AxumResponse;
use hyper::Request;
use hyper::Response;
use std::any::type_name;
use std::error::Error;
use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tower::{Layer, Service};

#[allow(clippy::type_complexity)]
/// Layer that applies [`ProxyService`] to [`Request`].
pub struct ProxyLayer<
    TClient,
    TBody,
    TRespBody,
    ShouldProxyCallback,
    OnProxyErrorCallback,
    OnProxyRequestCallback,
    OnProxyResponseCallback,
    TExtension = (),
> {
    client: TClient,
    shared: Arc<
        Shared<
            TBody,
            TRespBody,
            ShouldProxyCallback,
            OnProxyErrorCallback,
            OnProxyRequestCallback,
            OnProxyResponseCallback,
            TExtension,
        >,
    >,
}
impl<
        TClient,
        TBody,
        TRespBody,
        ShouldProxyCallback,
        OnProxyErrorCallback,
        OnProxyRequestCallback,
        OnProxyResponseCallback,
        TExtension,
    > Debug
    for ProxyLayer<
        TClient,
        TBody,
        TRespBody,
        ShouldProxyCallback,
        OnProxyErrorCallback,
        OnProxyRequestCallback,
        OnProxyResponseCallback,
        TExtension,
    >
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProxyLayer")
            .field("client", &format_args!("{}", type_name::<TClient>()))
            .field("shared", &self.shared)
            .finish()
    }
}

impl<
        TClient,
        TBody,
        TRespBody,
        ShouldProxyCallback,
        OnProxyErrorCallback,
        OnProxyRequestCallback,
        OnProxyResponseCallback,
        TExtension,
    > Clone
    for ProxyLayer<
        TClient,
        TBody,
        TRespBody,
        ShouldProxyCallback,
        OnProxyErrorCallback,
        OnProxyRequestCallback,
        OnProxyResponseCallback,
        TExtension,
    >
where
    TClient: Clone,
{
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            shared: Arc::clone(&self.shared),
        }
    }
}

#[allow(clippy::type_complexity)]
impl<
        TClient,
        TBody,
        TRespBody,
        ShouldProxyCallback,
        OnProxyErrorCallback,
        OnProxyRequestCallback,
        OnProxyResponseCallback,
    >
    ProxyLayer<
        TClient,
        TBody,
        TRespBody,
        ShouldProxyCallback,
        OnProxyErrorCallback,
        OnProxyRequestCallback,
        OnProxyResponseCallback,
    >
{
    /// Creates new [`ProxyLayer`].
    /// Returns Error if destination does not have [`hyper::http::uri::Scheme`] or [`hyper::http::uri::Authority`]
    pub fn new(
        client: TClient,
        destination: impl Into<String>,
        on_proxy_error: OnProxyErrorCallback,
        on_proxy_request: OnProxyRequestCallback,
        on_proxy_response: OnProxyResponseCallback,
        should_proxy: ShouldProxyCallback,
    ) -> Result<
        ProxyLayer<
            TClient,
            TBody,
            TRespBody,
            ShouldProxyCallback,
            OnProxyErrorCallback,
            OnProxyRequestCallback,
            OnProxyResponseCallback,
        >,
        String,
    >
    where
        TClient: Service<Request<TBody>, Response = Response<TRespBody>>,
        TClient: Clone + Send + Sync + 'static,
        <TClient as Service<Request<TBody>>>::Future: Send + 'static,
        <TClient as Service<Request<TBody>>>::Error: Into<Box<(dyn Error + Send + Sync + 'static)>>,
        ShouldProxyCallback: for<'any> Fn(
                &'any Request<TBody>,
                &'any (),
            ) -> Pin<Box<dyn Future<Output = bool> + Send + 'any>>
            + Send
            + Sync
            + 'static,
        OnProxyErrorCallback: Fn(ProxyError, &()) -> AxumResponse + Send + Sync + 'static,
        OnProxyRequestCallback: Fn(&Request<TBody>, &()) + Send + Sync + 'static,
        OnProxyResponseCallback: Fn(&Response<TRespBody>, &()) + Send + Sync + 'static,
        TBody: Sync + Send + 'static,
        TRespBody: HttpBody<Data = Bytes> + Sync + Send + 'static,
        TRespBody::Error: Into<Box<(dyn Error + Send + Sync + 'static)>>,
    {
        let shared = Shared::new_with_ext(
            destination,
            should_proxy,
            on_proxy_error,
            on_proxy_request,
            on_proxy_response,
        )?;

        Ok(Self {
            shared: Arc::new(shared),
            client,
        })
    }

    /// Creates new [`ProxyLayer`] with set [`TExtension`].
    /// Mostly this is needed to remove a need for extracting [`TExtension`] in every callback
    /// so it is extracted once and passed as reference to each callback.\
    /// Returns Error if destination does not have [`hyper::http::uri::Scheme`] or [`hyper::http::uri::Authority`]
    pub fn new_with_ext<TExtension>(
        client: TClient,
        destination: impl Into<String>,
        on_proxy_error: OnProxyErrorCallback,
        on_proxy_request: OnProxyRequestCallback,
        on_proxy_response: OnProxyResponseCallback,
        should_proxy: ShouldProxyCallback,
    ) -> Result<
        ProxyLayer<
            TClient,
            TBody,
            TRespBody,
            ShouldProxyCallback,
            OnProxyErrorCallback,
            OnProxyRequestCallback,
            OnProxyResponseCallback,
            TExtension,
        >,
        String,
    >
    where
        TClient: Service<Request<TBody>, Response = Response<TRespBody>>,
        TClient: Clone + Send + Sync + 'static,
        <TClient as Service<Request<TBody>>>::Future: Send + 'static,
        <TClient as Service<Request<TBody>>>::Error: Into<Box<(dyn Error + Send + Sync + 'static)>>,
        TExtension: Default + Clone + Send + Sync + 'static,
        ShouldProxyCallback: for<'any> Fn(
                &'any Request<TBody>,
                &'any TExtension,
            ) -> Pin<Box<dyn Future<Output = bool> + Send + 'any>>
            + Send
            + Sync
            + 'static,
        OnProxyErrorCallback: Fn(ProxyError, &TExtension) -> AxumResponse + Send + Sync + 'static,
        OnProxyRequestCallback: Fn(&Request<TBody>, &TExtension) + Send + Sync + 'static,
        OnProxyResponseCallback: Fn(&Response<TRespBody>, &TExtension) + Send + Sync + 'static,
        TBody: Sync + Send + 'static,
        TRespBody: HttpBody<Data = Bytes> + Sync + Send + 'static,
        TRespBody::Error: Into<Box<(dyn Error + Send + Sync + 'static)>>,
    {
        let shared = Shared::new_with_ext(
            destination,
            should_proxy,
            on_proxy_error,
            on_proxy_request,
            on_proxy_response,
        )?;

        Ok(ProxyLayer {
            shared: Arc::new(shared),
            client,
        })
    }
}

impl<
        S,
        TClient,
        TBody,
        TRespBody,
        ShouldProxyCallback,
        OnProxyErrorCallback,
        OnProxyRequestCallback,
        OnProxyResponseCallback,
        TExtension,
    > Layer<S>
    for ProxyLayer<
        TClient,
        TBody,
        TRespBody,
        ShouldProxyCallback,
        OnProxyErrorCallback,
        OnProxyRequestCallback,
        OnProxyResponseCallback,
        TExtension,
    >
where
    TClient: Clone,
{
    type Service = ProxyService<
        TClient,
        TBody,
        TRespBody,
        ShouldProxyCallback,
        OnProxyErrorCallback,
        OnProxyRequestCallback,
        OnProxyResponseCallback,
        S,
        TExtension,
    >;

    fn layer(&self, inner: S) -> Self::Service {
        ProxyService {
            shared: Arc::clone(&self.shared),
            client: self.client.clone(),
            inner,
            poll_error: None,
        }
    }
}
