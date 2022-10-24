use crate::{
    application::{app::shutdown_signal, tls_config::RemoteAddr},
    error::Result,
};
use async_stream::try_stream;
use axum::Router;
use futures_util::{stream::Stream, StreamExt, TryStreamExt};
use hyper::{server::accept, Server};
use std::{future::ready, net::SocketAddr};
use tokio::net::{TcpListener, TcpStream};
use tokio_native_tls::{
    native_tls::{self, Identity},
    TlsAcceptor, TlsStream,
};
use tokio_stream::wrappers::TcpListenerStream;
use tracing::{error, info};

pub(super) async fn run_service(
    socket: &SocketAddr,
    router: Router,
    pem: &[u8],
    key: &[u8],
) -> Result<()> {
    let identity = Identity::from_pkcs8(pem, key)?;
    let acceptor = TlsAcceptor::from(native_tls::TlsAcceptor::new(identity)?);

    let stream = bind_tls_stream(socket, acceptor).await?;
    let incoming = accept::from_stream(stream);

    let app = router.into_make_service_with_connect_info::<RemoteAddr>();
    let server = Server::builder(incoming).serve(app);

    info!(target: "server", "Started: https://{socket}");

    Ok(server.with_graceful_shutdown(shutdown_signal()).await?)
}

async fn bind_tls_stream(
    socket: &SocketAddr,
    acceptor: TlsAcceptor,
) -> Result<impl Stream<Item = Result<TlsStream<TcpStream>>>> {
    let listener = TcpListener::bind(socket).await?;
    let mut stream = TcpListenerStream::new(listener);

    let ret = try_stream! {
        while let Some(stream) = stream.try_next().await? {
            yield acceptor.accept(stream).await?;
        }
    }
    .filter(|tls_stream| {
        let ret = if let Err(error) = tls_stream {
            error!("Got error on incoming: `{error}`.");
            false
        } else {
            true
        };

        ready(ret)
    });

    Ok(ret)
}
