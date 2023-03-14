use crate::middleware::proxy_layer::error::ProxyError;
use axum::body::{Bytes, HttpBody};
use axum::response::{IntoResponse, Response as AxumResponse};
use core::any::type_name;
use hyper::http::uri::PathAndQuery;
use hyper::Request;
use hyper::Response;
use hyper::Uri;
use std::error::Error;
use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::str::FromStr;
use tower::{Service, ServiceExt};

pub(crate) struct Shared<
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
    destination: Uri,
    should_proxy: ShouldProxyCallback,
    on_proxy_error: OnProxyErrorCallback,
    on_proxy_request: OnProxyRequestCallback,
    on_proxy_response: OnProxyResponseCallback,
    phantom: PhantomData<(TExtension, TBody, TRespBody)>,
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
    for Shared<
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
        f.debug_struct("Shared")
            .field("client", &format_args!("{}", type_name::<TClient>()))
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
        TClient,
        TBody,
        TRespBody,
        ShouldProxyCallback,
        OnProxyErrorCallback,
        OnProxyRequestCallback,
        OnProxyResponseCallback,
    >
    Shared<
        TClient,
        TBody,
        TRespBody,
        ShouldProxyCallback,
        OnProxyErrorCallback,
        OnProxyRequestCallback,
        OnProxyResponseCallback,
    >
{
    pub(crate) fn new_with_ext<TExtension>(
        client: TClient,
        destination: impl Into<String>,
        should_proxy: ShouldProxyCallback,
        on_proxy_error: OnProxyErrorCallback,
        on_proxy_request: OnProxyRequestCallback,
        on_proxy_response: OnProxyResponseCallback,
    ) -> Result<
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
        let destination = Uri::from_str(&destination.into()).map_err(|err| err.to_string())?;

        let _ = destination
            .scheme()
            .ok_or("destination Uri has no scheme!".to_string())?;
        let _ = destination
            .authority()
            .ok_or("destination Uri has no authority!".to_string())?;

        let layer = Shared::<
            TClient,
            TBody,
            TRespBody,
            ShouldProxyCallback,
            OnProxyErrorCallback,
            OnProxyRequestCallback,
            OnProxyResponseCallback,
            TExtension,
        > {
            client,
            destination,
            should_proxy,
            on_proxy_error,
            on_proxy_request,
            on_proxy_response,
            phantom: PhantomData::default(),
        };

        Ok(layer)
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
    >
    Shared<
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
    pub(crate) async fn proxy(
        &self,
        mut req: Request<TBody>,
        extension: TExtension,
    ) -> AxumResponse {
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
                let client = self.client.clone();
                let result = Self::send_reqeust(client, req).await;

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

    pub(crate) async fn should_proxy(
        &self,
        request: &Request<TBody>,
        extension: &TExtension,
    ) -> bool {
        (self.should_proxy)(request, extension).await
    }

    pub(crate) fn get_extension(&self, request: &Request<TBody>) -> TExtension {
        request
            .extensions()
            .get::<TExtension>()
            .cloned()
            .unwrap_or_default()
    }

    async fn send_reqeust(
        mut service: TClient,
        request: Request<TBody>,
    ) -> Result<Response<TRespBody>, ProxyError> {
        let ready_svc = service.ready().await;

        match ready_svc {
            Ok(client) => Ok(client
                .call(request)
                .await
                .map_err(|err| ProxyError::SendRequest(err.into()))?),
            Err(err) => Err(ProxyError::SendRequest(err.into())),
        }
    }
}
