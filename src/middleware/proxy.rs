// FIXME(kos): Proxy should be already implemented:
//             https://github.com/felipenoris/hyper-reverse-proxy
//             Does not it fit your needs?

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
        // FIXME: this may leak private information to the client.
        // Be careful of what gets sent in `err`.
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed with: {}", err),
        )
            .into_response(),
    }
}

type Client = hyper::client::Client<HttpConnector, Body>;

/// Layer that applies [`Proxy`].
#[derive(Clone, Debug)]
pub struct ProxyLayer<F> {
    client: Client,
    destination: String,
    should_be_proxied_fn: F,
}

impl<F> ProxyLayer<F> {
    // TODO(kos): For ergonomic purposes, it would be nice to provide multiple
    //            constructor function (or even a builder), where, by default,
    //            the `Client` is created automatically, while still may be
    //            specified a custom `Client` if the caller needs.
    //            Alternatively consider introducing second constructor `new_with_client()`.
    /// Creates new [`ProxyLayer`]
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

/// Middleware that takes: Fn(&Request<Body>) -> bool which decides if to proxy request or not, and proxy it to destination using client
#[derive(Clone, Debug)]
pub struct Proxy<F, S> {
    client: Client,
    destination: String,
    inner: S,
    should_be_proxied_fn: F,
}

impl<F, S> Proxy<F, S> {
    /// Creates new [`Proxy`] with given fn to decide if to proxy request, destination where to proxy request and client which will be used to proxy
    pub fn new(service: S, should_be_proxied_fn: F, destination: String, client: Client) -> Self {
        Self {
            inner: service,
            should_be_proxied_fn,
            destination,
            client,
        }
    }

}

// TODO(kos): Consider using `F: FnMut` as, the `call()` method accepts `&mut`
//            anyway.
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
    /// Response future returned by [`Proxy`] middleware
    pub struct ResponseFuture<F> {
        #[pin]
        kind: FutureType<F>,
    }
}

impl<F> ResponseFuture<F> {
    pub(crate) fn future(future: F) -> Self {
        Self {
            kind: FutureType::Axum { future },
        }
    }

    pub(crate) fn hyper(future: hyper::client::ResponseFuture) -> Self {
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
            // TODO(kos): Consider using `future::ready!()` macro:
            //            ```rust
            //            Poll::Ready(Ok(handle_result(task::ready!(future.poll(cx))))
            //            ```
            //            https://doc.rust-lang.org/stable/std/task/macro.ready.html
            FutureProject::Hyper { future } => match future.poll(cx) {
                Poll::Ready(v) => Poll::Ready(Ok(handle_result(v))),
                Poll::Pending => Poll::Pending,
            },
        }
    }
}
