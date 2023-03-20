use crate::middleware::ProxyError;
use axum::response::IntoResponse;
use bytes::Bytes;
use core::any::type_name;
use hyper::body::HttpBody;
use hyper::http::uri::PathAndQuery;
use hyper::service::Service;
use hyper::{Request, Response, Uri};
use std::error::Error;
use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::str::FromStr;

pub(crate) struct Shared<
    TBody,
    TRespBody,
    ShouldProxyCallback,
    OnProxyErrorCallback,
    OnProxyRequestCallback,
    OnProxyResponseCallback,
    TExtension = (),
> {
    pub(crate) destination: Uri,
    pub(crate) should_proxy: ShouldProxyCallback,
    pub(crate) on_proxy_error: OnProxyErrorCallback,
    pub(crate) on_proxy_request: OnProxyRequestCallback,
    pub(crate) on_proxy_response: OnProxyResponseCallback,
    pub(crate) phantom: PhantomData<(TExtension, TBody, TRespBody)>,
}

impl<
        TBody,
        TRespBody,
        ShouldProxyCallback,
        OnProxyErrorCallback,
        OnProxyRequestCallback,
        OnProxyResponseCallback,
        TExtension,
    > Debug
    for Shared<
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
        f.debug_struct("Shared")
            .field("destination", &self.destination)
            .field(
                "on_proxy_error",
                &format_args!("{}", type_name::<OnProxyErrorCallback>()),
            )
            .field(
                "on_proxy_request",
                &format_args!("{}", type_name::<OnProxyRequestCallback>()),
            )
            .field(
                "on_proxy_response",
                &format_args!("{}", type_name::<OnProxyResponseCallback>()),
            )
            .field("extension", &format_args!("{}", type_name::<TExtension>()))
            .finish()
    }
}

#[allow(clippy::type_complexity)]
impl<
        TBody,
        TRespBody,
        ShouldProxyCallback,
        OnProxyErrorCallback,
        OnProxyRequestCallback,
        OnProxyResponseCallback,
    >
    Shared<
        TBody,
        TRespBody,
        ShouldProxyCallback,
        OnProxyErrorCallback,
        OnProxyRequestCallback,
        OnProxyResponseCallback,
    >
{
    pub(crate) fn new_with_ext<TExtension>(
        destination: impl Into<String>,
        should_proxy: ShouldProxyCallback,
        on_proxy_error: OnProxyErrorCallback,
        on_proxy_request: OnProxyRequestCallback,
        on_proxy_response: OnProxyResponseCallback,
    ) -> Result<
        Shared<
            TBody,
            TRespBody,
            ShouldProxyCallback,
            OnProxyErrorCallback,
            OnProxyRequestCallback,
            OnProxyResponseCallback,
            TExtension,
        >,
        String,
    > {
        let destination = Uri::from_str(&destination.into()).map_err(|err| err.to_string())?;

        let _ = destination
            .scheme()
            .ok_or("destination Uri has no scheme!".to_string())?;
        let _ = destination
            .authority()
            .ok_or("destination Uri has no authority!".to_string())?;

        let shared = Shared::<
            TBody,
            TRespBody,
            ShouldProxyCallback,
            OnProxyErrorCallback,
            OnProxyRequestCallback,
            OnProxyResponseCallback,
            TExtension,
        > {
            destination,
            should_proxy,
            on_proxy_error,
            on_proxy_request,
            on_proxy_response,
            phantom: PhantomData::default(),
        };

        Ok(shared)
    }
}

impl<
        TBody,
        TRespBody,
        ShouldProxyCallback,
        OnProxyErrorCallback,
        OnProxyRequestCallback,
        OnProxyResponseCallback,
        TExtension,
    >
    Shared<
        TBody,
        TRespBody,
        ShouldProxyCallback,
        OnProxyErrorCallback,
        OnProxyRequestCallback,
        OnProxyResponseCallback,
        TExtension,
    >
where
    TExtension: Default + Clone + Send + Sync + 'static,
    ShouldProxyCallback: for<'any> Fn(
            &'any Request<TBody>,
            &'any TExtension,
        ) -> Pin<Box<dyn Future<Output = bool> + Send + 'any>>
        + Send
        + Sync
        + 'static,
    OnProxyErrorCallback:
        Fn(ProxyError, &TExtension) -> axum::response::Response + Send + Sync + 'static,
    OnProxyRequestCallback: Fn(&Request<TBody>, &TExtension) + Send + Sync + 'static,
    OnProxyResponseCallback: Fn(&Response<TRespBody>, &TExtension) + Send + Sync + 'static,
    TBody: Sync + Send + 'static,
    TRespBody: HttpBody<Data = Bytes> + Sync + Send + 'static,
    TRespBody::Error: Into<Box<(dyn Error + Send + Sync + 'static)>>,
{
    pub(crate) async fn proxy<TClient>(
        &self,
        mut req: Request<TBody>,
        client: TClient,
        extension: TExtension,
        poll_error: Option<Box<(dyn Error + Send + Sync + 'static)>>,
    ) -> axum::response::Response
    where
        TClient: Service<Request<TBody>, Response = Response<TRespBody>>,
        TClient: Clone + Send + Sync + 'static,
        <TClient as Service<Request<TBody>>>::Future: Send + 'static,
        <TClient as Service<Request<TBody>>>::Error:
            Into<Box<(dyn Error + Send + Sync + 'static)>> + Send,
    {
        if let Some(err) = poll_error {
            return (self.on_proxy_error)(ProxyError::SendRequest(err), &extension);
        }

        let build_uri = |req: &Request<TBody>| {
            let p_and_q = req
                .uri()
                .path_and_query()
                .map_or_else(|| req.uri().path(), PathAndQuery::as_str);

            let destination_parts = self.destination.clone().into_parts();

            #[allow(clippy::expect_used)]
            // SAFETY: Checked on [`Shared::new`]
            let authority = destination_parts
                .authority
                .expect("Destination uri must have [Authority]");

            #[allow(clippy::expect_used)]
            // SAFETY: Checked on [`Shared::new`]
            let scheme = destination_parts
                .scheme
                .expect("Destination uri must have [Scheme]");

            Uri::builder()
                .authority(authority)
                .scheme(scheme)
                .path_and_query(p_and_q)
                .build()
                .map_err(ProxyError::UriBuilder)
        };

        match build_uri(&req) {
            Ok(new_uri) => {
                *req.uri_mut() = new_uri;

                (self.on_proxy_request)(&req, &extension);
                let result = send_request(client, req).await;

                match result {
                    Ok(response) => {
                        (self.on_proxy_response)(&response, &extension);
                        response.into_response()
                    }
                    Err(err) => (self.on_proxy_error)(err, &extension),
                }
            }
            Err(err) => (self.on_proxy_error)(err, &extension),
        }
    }
}

pub(crate) fn get_extension<TBody, TExtension>(request: &Request<TBody>) -> TExtension
where
    TExtension: Default + Clone + Send + Sync + 'static,
{
    request
        .extensions()
        .get::<TExtension>()
        .cloned()
        .unwrap_or_default()
}

#[allow(clippy::needless_question_mark)]
async fn send_request<TClient, TBody, TRespBody>(
    mut service: TClient,
    request: Request<TBody>,
) -> Result<Response<TRespBody>, ProxyError>
where
    TClient: Service<Request<TBody>, Response = Response<TRespBody>>,
    TClient: Clone + Send + Sync + 'static,
    <TClient as Service<Request<TBody>>>::Future: Send + 'static,
    <TClient as Service<Request<TBody>>>::Error:
        Into<Box<(dyn Error + Send + Sync + 'static)>> + Send,
{
    Ok(service
        .call(request)
        .await
        .map_err(|err| ProxyError::SendRequest(err.into()))?)
}
