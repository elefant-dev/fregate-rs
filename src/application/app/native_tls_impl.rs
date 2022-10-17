use super::{shutdown_signal, Result};
use axum::extract::connect_info::IntoMakeServiceWithConnectInfo;
use axum::Router;
use futures_util::future::poll_fn;
use hyper::server::{
    accept::Accept,
    conn::{AddrIncoming, Http},
};
use std::{net::SocketAddr, pin::Pin, sync::Arc};
use tokio::{net::TcpListener, select};
use tokio_native_tls::{
    native_tls::{self, Identity},
    TlsAcceptor,
};
use tower::MakeService;
use tracing::error;

pub(super) async fn run_tls_service(
    socket: SocketAddr,
    rest: Router,
    pem: &[u8],
    key: &[u8],
) -> Result<()> {
    let identify = Identity::from_pkcs8(pem, key)?;
    let acceptor = TlsAcceptor::from(native_tls::TlsAcceptor::builder(identify).build()?);

    let listener = AddrIncoming::from_listener(TcpListener::bind(socket).await?)?;
    let protocol = Arc::new(Http::new());

    let application = rest.into_make_service_with_connect_info();

    select! {
        _ = shutdown_signal() => Ok(()),
        ret = tls_loop(listener, acceptor, protocol, application) => ret,
    }
}

async fn tls_loop(
    mut listener: AddrIncoming,
    acceptor: TlsAcceptor,
    protocol: Arc<Http>,
    mut application: IntoMakeServiceWithConnectInfo<Router, SocketAddr>,
) -> Result<()> {
    loop {
        // Always [`Option::Some`].
        // https://docs.rs/hyper/0.14.20/src/hyper/server/tcp.rs.html#166
        let stream = poll_fn(|cx| Pin::new(&mut listener).poll_accept(cx))
            .await
            .unwrap()?;
        let acceptor = acceptor.clone();
        let protocol = protocol.clone();
        let service = application.make_service(&stream);

        tokio::task::spawn(async move {
            match acceptor.accept(stream).await {
                Ok(tls_stream) => match service.await {
                    Ok(service) => match protocol.serve_connection(tls_stream, service).await {
                        Ok(()) => {}
                        Err(error) => {
                            error!("Http::serve_connection got error: `{error}`.")
                        }
                    },
                    Err(_infallible) => error!("service.await error"),
                },
                Err(error) => error!("TlsAcceptor::accept got error: `{error}`."),
            }
        });
    }
}
