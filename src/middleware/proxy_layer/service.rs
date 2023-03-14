//! Middleware that runs [`ShouldProxyCallback`] and on `true` proxy [`Request`] to a new destination otherwise pass [`Request`] to next handler.\
//! See in [examples](https://github.com/elefant-dev/fregate-rs/blob/main/examples/proxy-layer/src/main.rs) how it might be used
use crate::middleware::proxy_layer::error::ProxyError;
use crate::middleware::proxy_layer::shared::Shared;
use axum::body::{Bytes, HttpBody};
use axum::response::{IntoResponse, Response as AxumResponse};
use hyper::Request;
use hyper::Response;
use std::convert::Infallible;
use std::error::Error;
use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tower::Service;

#[allow(clippy::type_complexity)]
/// Middleware that runs [`ShouldProxyCallback`] and on `true` proxy [`Request`] to a new destination otherwise pass [`Request`] to next handler.
pub struct ProxyService<
    TClient,
    TBody,
    TRespBody,
    ShouldProxyCallback,
    OnProxyErrorCallback,
    OnProxyRequestCallback,
    OnProxyResponseCallback,
    TService,
    TExtension = (),
> {
    pub(crate) shared: Arc<
        Shared<
            TClient,
            TBody,
            TRespBody,
            ShouldProxyCallback,
            OnProxyErrorCallback,
            OnProxyRequestCallback,
            OnProxyResponseCallback,
            TExtension,
        >,
    >,
    pub(crate) inner: TService,
}

impl<
        TClient,
        TBody,
        TRespBody,
        ShouldProxyCallback,
        OnProxyErrorCallback,
        OnProxyRequestCallback,
        OnProxyResponseCallback,
        TService,
        TExtension,
    > Clone
    for ProxyService<
        TClient,
        TBody,
        TRespBody,
        ShouldProxyCallback,
        OnProxyErrorCallback,
        OnProxyRequestCallback,
        OnProxyResponseCallback,
        TService,
        TExtension,
    >
where
    TService: Clone,
{
    fn clone(&self) -> Self {
        Self {
            shared: Arc::clone(&self.shared),
            inner: self.inner.clone(),
        }
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
        TService,
        TExtension,
    > Debug
    for ProxyService<
        TClient,
        TBody,
        TRespBody,
        ShouldProxyCallback,
        OnProxyErrorCallback,
        OnProxyRequestCallback,
        OnProxyResponseCallback,
        TService,
        TExtension,
    >
where
    TService: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Proxy")
            .field("shared", &self.shared)
            .field("inner", &self.inner)
            .finish()
    }
}

impl<
        TBody,
        TRespBody,
        TClient,
        ShouldProxyCallback,
        OnProxyErrorCallback,
        OnProxyRequestCallback,
        OnProxyResponseCallback,
        TService,
        TExtension,
    > Service<Request<TBody>>
    for ProxyService<
        TClient,
        TBody,
        TRespBody,
        ShouldProxyCallback,
        OnProxyErrorCallback,
        OnProxyRequestCallback,
        OnProxyResponseCallback,
        TService,
        TExtension,
    >
where
    TClient: Service<Request<TBody>, Response = Response<TRespBody>>,
    TClient: Clone + Send + Sync + 'static,
    <TClient as Service<Request<TBody>>>::Future: Send + 'static,
    <TClient as Service<Request<TBody>>>::Error:
        Into<Box<(dyn Error + Send + Sync + 'static)>> + Send,
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
    TService: Service<Request<TBody>, Response = AxumResponse, Error = Infallible>
        + Clone
        + Send
        + 'static,
    TService::Future: Send + 'static,
{
    type Response = TService::Response;
    type Error = TService::Error;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<TBody>) -> Self::Future {
        let not_ready_inner_clone = self.inner.clone();
        let mut ready_inner_clone = std::mem::replace(&mut self.inner, not_ready_inner_clone);
        let shared = self.shared.clone();

        let future = async move {
            let extension = shared.get_extension(&request);

            if shared.should_proxy(&request, &extension).await {
                Ok(shared.proxy(request, extension).await.into_response())
            } else {
                match ready_inner_clone.call(request).await {
                    Ok(resp) => Ok(resp),
                    Err(_) => {
                        unreachable!("Service returns Infallible Error")
                    }
                }
            }
        };

        Box::pin(future)
    }
}
