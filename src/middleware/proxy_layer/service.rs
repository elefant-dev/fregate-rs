//! Middleware that runs [`ShouldProxyCallback`] and on `true` proxy [`Request`] to a new destination otherwise pass [`Request`] to next handler.\
//! See in [examples](https://github.com/elefant-dev/fregate-rs/blob/main/examples/proxy-layer/src/main.rs) how it might be used
use crate::middleware::proxy_layer::error::ProxyError;
use crate::middleware::proxy_layer::shared::{get_extension, Shared};
use axum::body::{Bytes, HttpBody};
use axum::response::{IntoResponse, Response as AxumResponse};
use hyper::Request;
use hyper::Response;
use std::any::type_name;
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
            TBody,
            TRespBody,
            ShouldProxyCallback,
            OnProxyErrorCallback,
            OnProxyRequestCallback,
            OnProxyResponseCallback,
            TExtension,
        >,
    >,
    pub(crate) client: TClient,
    pub(crate) inner: TService,
    pub(crate) poll_error: Option<Box<(dyn Error + Send + Sync + 'static)>>,
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
    TClient: Clone,
{
    fn clone(&self) -> Self {
        Self {
            shared: Arc::clone(&self.shared),
            inner: self.inner.clone(),
            client: self.client.clone(),
            poll_error: None,
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
            .field("client", &format_args!("{}", type_name::<TClient>()))
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
        )
            -> Pin<Box<dyn Future<Output = Result<bool, AxumResponse>> + Send + 'any>>
        + Send
        + Sync
        + 'static,
    OnProxyErrorCallback: Fn(ProxyError, &TExtension) -> AxumResponse + Send + Sync + 'static,
    OnProxyRequestCallback: Fn(&mut Request<TBody>, &TExtension) + Send + Sync + 'static,
    OnProxyResponseCallback: Fn(&mut Response<TRespBody>, &TExtension) + Send + Sync + 'static,
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
        match (self.client.poll_ready(cx), self.inner.poll_ready(cx)) {
            (Poll::Ready(Ok(())), Poll::Ready(Ok(()))) => Poll::Ready(Ok(())),
            (Poll::Ready(Err(e)), _) => {
                self.poll_error.replace(e.into());
                Poll::Ready(Ok(()))
            }
            (_, Poll::Ready(Err(e))) => Poll::Ready(Err(e)),
            _ => Poll::Pending,
        }
    }

    fn call(&mut self, request: Request<TBody>) -> Self::Future {
        let not_ready_inner_clone = self.inner.clone();
        let mut ready_inner_clone = std::mem::replace(&mut self.inner, not_ready_inner_clone);

        let client = self.client.clone();
        let ready_client = std::mem::replace(&mut self.client, client);

        let shared = self.shared.clone();
        let poll_error = self.poll_error.take();

        let future = async move {
            let extension = get_extension(&request);

            match (shared.should_proxy)(&request, &extension).await {
                Ok(true) => Ok(shared
                    .proxy(request, ready_client, extension, poll_error)
                    .await
                    .into_response()),
                Ok(false) => match ready_inner_clone.call(request).await {
                    Ok(resp) => Ok(resp),
                    Err(err) => Err(err),
                },
                Err(err) => Ok(err),
            }
        };

        Box::pin(future)
    }
}
