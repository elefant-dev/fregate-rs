use axum::body::BoxBody;
use axum::response::IntoResponse;
use hyper::{
    client::HttpConnector,
    http::{Request, Response},
    Body, Uri,
};
use pin_project_lite::pin_project;
use std::marker::PhantomData;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower::{Layer, Service};
use tracing::info;

type Client = hyper::client::Client<HttpConnector, Body>;

#[derive(Clone)]
pub struct ProxyLayer<F, B> {
    client: Client,
    destination: String,
    f: F,
    phantom: PhantomData<B>,
}

impl<F, B> ProxyLayer<F, B>
where
    F: Fn(&Request<B>) -> bool,
{
    pub fn new(f: F, client: Client, destination: &str) -> Self {
        Self {
            f,
            phantom: PhantomData::default(),
            client,
            destination: destination.to_owned(),
        }
    }
}

impl<S, F, B> Layer<S> for ProxyLayer<F, B>
where
    F: Clone,
{
    type Service = Proxy<S, F>;

    fn layer(&self, inner: S) -> Self::Service {
        Proxy::new(
            inner,
            self.f.clone(),
            self.destination.clone(),
            self.client.clone(),
        )
    }
}

#[derive(Clone)]
pub struct Proxy<S, F> {
    client: Client,
    destination: String,
    inner: S,
    f: F,
}

impl<S, F> Proxy<S, F> {
    pub fn new(service: S, f: F, destination: String, client: Client) -> Self {
        Self {
            inner: service,
            f,
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

impl<F, S> Service<Request<Body>> for Proxy<S, F>
where
    S: Service<Request<Body>, Response = Response<BoxBody>>,
    F: Fn(&Request<Body>) -> bool,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = ResponseFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match self.inner.poll_ready(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(r) => Poll::Ready(r.map_err(Into::into)),
        }
    }

    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
        if (self.f)(&req) {
            let path_query = req
                .uri()
                .path_and_query()
                .map(|v| v.as_str())
                .unwrap_or_else(|| req.uri().path());

            let uri = format!("{}{}", self.destination, path_query);
            *req.uri_mut() = Uri::try_from(uri).unwrap();

            info!("Proxy Call");
            ResponseFuture::hyper(self.client.call(req))
        } else {
            info!("Local Resolver Call");
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

impl<F, E> Future for ResponseFuture<F>
where
    F: Future<Output = Result<Response<BoxBody>, E>>,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.project().kind.project() {
            FutureProject::Axum { future } => future.poll(cx),
            FutureProject::Hyper { future } => match future.poll(cx) {
                Poll::Ready(v) => {
                    // TODO! : Add Error Handling
                    Poll::Ready(Ok(v.map(axum::body::boxed).unwrap().into_response()))
                }
                Poll::Pending => Poll::Pending,
            },
        }
    }
}
