use crate::{
    application::{app::shutdown_signal, tls_config::RemoteAddr},
    error::{Error, Result},
};
use async_stream::stream;
use axum::Router;
use futures_util::{
    stream::{FuturesUnordered, Stream},
    StreamExt, TryStreamExt,
};
use hyper::{server::accept, Server};
use std::{net::SocketAddr, sync::Arc};
use tokio::{
    net::{TcpListener, TcpStream},
    select,
    task::JoinHandle,
};
use tokio_native_tls::{
    native_tls::{self, Identity},
    TlsAcceptor, TlsStream,
};
use tokio_stream::wrappers::TcpListenerStream;
use tracing::{info, warn};

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
    let mut tcp_stream = TcpListenerStream::new(listener);

    let acceptor = Arc::new(acceptor);
    let ret = stream! {
        let mut tasks = FuturesUnordered::new();

        loop {
            match fetch_tls_handle_commands(&mut tcp_stream, &mut tasks).await {
                Ok(TlsHandleCommands::TcpStream(tcp_stream)) => {
                    let acceptor = acceptor.clone();
                    tasks.push(tokio::task::spawn(async move { Ok::<_, Error>(acceptor.accept(tcp_stream).await?) }));
                },
                Ok(TlsHandleCommands::TlsStream(tls_stream)) => yield Ok(tls_stream),
                Ok(TlsHandleCommands::Break) => break,
                Err(error) => warn!("Got error on incoming: `{error}`."),
            }
        }
    };

    Ok(ret)
}

enum TlsHandleCommands {
    TcpStream(TcpStream),
    TlsStream(TlsStream<TcpStream>),
    Break,
}

async fn fetch_tls_handle_commands(
    tcp_stream: &mut TcpListenerStream,
    tasks: &mut FuturesUnordered<JoinHandle<Result<TlsStream<TcpStream>>>>,
) -> Result<TlsHandleCommands> {
    let ret = if tasks.is_empty() {
        match tcp_stream.try_next().await? {
            None => TlsHandleCommands::Break,
            Some(tcp_stream) => TlsHandleCommands::TcpStream(tcp_stream),
        }
    } else {
        select! {
            tcp_stream = tcp_stream.try_next() => {
                tcp_stream?.map(TlsHandleCommands::TcpStream).unwrap_or(TlsHandleCommands::Break)
            }
            tls_stream = tasks.next() => {
                let tls_stream = tls_stream.expect("FuturesUnordered stream can't be closed in ordinary circumstances")??;
                TlsHandleCommands::TlsStream(tls_stream)
            }
        }
    };

    Ok(ret)
}
