use axum::{body::BoxBody, http::StatusCode, response::IntoResponse, BoxError};
use bytes::Bytes;
use hyper::{
    client::HttpConnector,
    http::{Request, Response},
    Body, Uri,
};
use pin_project_lite::pin_project;
use std::{
    error::Error,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower::{Layer, Service};
use tracing::error;

fn handle_result<B, E>(result: Result<Response<B>, E>) -> Response<BoxBody>
where
    E: Error,
    B: http_body::Body<Data = Bytes> + Send + 'static,
    B::Error: Into<BoxError>,
{
    match result {
        Ok(resp) => resp.into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed with: {}", err),
        )
            .into_response(),
    }
}

type Client = hyper::client::Client<HttpConnector, Body>;

#[derive(Clone, Debug)]
pub struct ProxyLayer<F> {
    client: Client,
    destination: String,
    should_be_proxied_fn: F,
}

impl<F> ProxyLayer<F> {
    pub fn new<B>(should_be_proxied_fn: F, client: Client, destination: &str) -> Self
    where
        F: Fn(&Request<B>) -> bool,
    {
        Self {
            should_be_proxied_fn,
            client,
            destination: destination.to_owned(),
        }
    }
}

impl<F, S> Layer<S> for ProxyLayer<F>
where
    F: Clone,
{
    type Service = Proxy<F, S>;

    fn layer(&self, inner: S) -> Self::Service {
        Proxy::new(
            inner,
            self.should_be_proxied_fn.clone(),
            self.destination.clone(),
            self.client.clone(),
        )
    }
}

#[derive(Clone, Debug)]
pub struct Proxy<F, S> {
    client: Client,
    destination: String,
    inner: S,
    should_be_proxied_fn: F,
}

impl<F, S> Proxy<F, S> {
    pub fn new(service: S, should_be_proxied_fn: F, destination: String, client: Client) -> Self {
        Self {
            inner: service,
            should_be_proxied_fn,
            destination,
            client,
        }
    }

    pub fn get_ref(&self) -> &S {
        &self.inner
    }

    pub fn get_mut(&mut self) -> &mut S {
        &mut self.inner
    }

    pub fn into_inner(self) -> S {
        self.inner
    }
}

impl<F, S> Service<Request<Body>> for Proxy<F, S>
where
    F: Fn(&Request<Body>) -> bool,
    S: Service<Request<Body>, Response = Response<BoxBody>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = ResponseFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
        if (self.should_be_proxied_fn)(&req) {
            let path_query = req
                .uri()
                .path_and_query()
                .map(|v| v.as_str())
                .unwrap_or_else(|| req.uri().path());

            let uri = format!("{}{}", self.destination, path_query);

            match Uri::try_from(uri) {
                Ok(new_uri) => {
                    *req.uri_mut() = new_uri;
                    ResponseFuture::hyper(self.client.call(req))
                }
                Err(err) => {
                    error!("Failed to proxy request to {} with error: {err} going to use local Handler for {} endpoint", self.destination, req.uri());
                    ResponseFuture::future(self.inner.call(req))
                }
            }
        } else {
            ResponseFuture::future(self.inner.call(req))
        }
    }
}

pin_project! {
    pub struct ResponseFuture<F> {
        #[pin]
        kind: FutureType<F>,
    }
}

impl<F> ResponseFuture<F> {
    fn future(future: F) -> Self {
        Self {
            kind: FutureType::Axum { future },
        }
    }

    fn hyper(future: hyper::client::ResponseFuture) -> Self {
        Self {
            kind: FutureType::Hyper { future },
        }
    }
}

pin_project! {
    #[project = FutureProject]
    enum FutureType<F> {
        Axum {
            #[pin]
            future: F,
        },
        Hyper {
            #[pin]
            future: hyper::client::ResponseFuture,
        },
    }
}

impl<E, F> Future for ResponseFuture<F>
where
    F: Future<Output = Result<Response<BoxBody>, E>>,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.project().kind.project() {
            FutureProject::Axum { future } => future.poll(cx),
            FutureProject::Hyper { future } => match future.poll(cx) {
                Poll::Ready(v) => Poll::Ready(Ok(handle_result(v))),
                Poll::Pending => Poll::Pending,
            },
        }
    }
}
