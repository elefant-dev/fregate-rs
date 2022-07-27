use axum::{
    body::{self, Body},
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use hyper::upgrade::Upgraded;
use std::future::Future;
use tokio::net::TcpStream;
use tower::util::ServiceFn;

use fregate::application::Application;

type _Svs = ServiceFn<dyn FnOnce(Request<Body>) -> dyn Future<Output = Response>>;

#[tokio::main]
async fn main() {
    let router = Router::new().route("/", get(handler));

    // let service = tower::service_fn(move |req: Request<Body>| {
    //     let router = router.clone();
    //     async move {
    //         if req.method() == Method::CONNECT {
    //             proxy(req).await
    //         } else {
    //             router.oneshot(req).await.map_err(|err| match err {})
    //         }
    //     }
    // });

    let app = Application::builder()
        .telemetry(true)
        .rest_router(router)
        //        .service(service)
        .build();

    app.run().await.unwrap();
}

async fn handler() -> &'static str {
    "Hello, Proxy!"
}

async fn _proxy(req: Request<Body>) -> Result<Response, hyper::Error> {
    tracing::trace!(?req);

    if let Some(host_addr) = req.uri().authority().map(|auth| auth.to_string()) {
        tokio::task::spawn(async move {
            match hyper::upgrade::on(req).await {
                Ok(upgraded) => {
                    if let Err(e) = _tunnel(upgraded, host_addr).await {
                        tracing::warn!("server io error: {}", e);
                    };
                }
                Err(e) => tracing::warn!("upgrade error: {}", e),
            }
        });

        Ok(Response::new(body::boxed(body::Empty::new())))
    } else {
        tracing::warn!("CONNECT host is not socket addr: {:?}", req.uri());
        Ok((
            StatusCode::BAD_REQUEST,
            "CONNECT must be to a socket address",
        )
            .into_response())
    }
}

async fn _tunnel(mut upgraded: Upgraded, addr: String) -> std::io::Result<()> {
    let mut server = TcpStream::connect(addr).await?;

    let (from_client, from_server) =
        tokio::io::copy_bidirectional(&mut upgraded, &mut server).await?;

    tracing::debug!(
        "client wrote {} bytes and received {} bytes",
        from_client,
        from_server
    );

    Ok(())
}
