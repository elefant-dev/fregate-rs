#[cfg(all(feature = "use_native_tls", feature = "use_rustls"))]
compile_error!("native-tls and rustls cannot be used together");

#[cfg(not(all(
    feature = "tls",
    any(feature = "use_native_tls", feature = "use_rustls")
)))]
compile_error!("can't use tls flags directly");

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
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::{
    net::{TcpListener, TcpStream},
    select,
    task::JoinHandle,
    time::timeout,
};
use tokio_stream::wrappers::TcpListenerStream;
use tracing::{info, warn};

pub(crate) use reexport::*;

pub(super) async fn run_service(
    socket: &SocketAddr,
    router: Router,
    tls_handshake_timeout: Duration,
    pem: Vec<u8>,
    key: Vec<u8>,
) -> Result<()> {
    let acceptor = create_acceptor(&pem, &key)?;
    drop((pem, key));

    let stream = bind_tls_stream(socket, acceptor, tls_handshake_timeout).await?;
    let incoming = accept::from_stream(stream);

    let app = router.into_make_service_with_connect_info::<RemoteAddr>();
    let server = Server::builder(incoming).serve(app);

    info!(target: "server", "Started: https://{socket}");

    Ok(server.with_graceful_shutdown(shutdown_signal()).await?)
}

async fn bind_tls_stream(
    socket: &SocketAddr,
    acceptor: TlsAcceptor,
    tls_handshake_timeout: Duration,
) -> Result<impl Stream<Item = Result<TlsStream>>> {
    let listener = TcpListener::bind(socket).await?;
    let mut tcp_stream = TcpListenerStream::new(listener);

    let acceptor = Arc::new(acceptor);
    let ret = stream! {
        let mut tasks = FuturesUnordered::new();

        loop {
            match fetch_tls_handle_commands(&mut tcp_stream, &mut tasks).await {
                Ok(TlsHandleCommands::TcpStream(tcp_stream)) => {
                    let acceptor = acceptor.clone();
                    tasks.push(tokio::task::spawn(async move {
                        let ret = timeout(tls_handshake_timeout, acceptor.accept(tcp_stream))
                            .await
                            .map_err(|_| Error::TlsHandshakeTimeout)??
                            .into();
                        Ok::<_, Error>(ret)
                    }));
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
    TlsStream(TlsStream),
    Break,
}

async fn fetch_tls_handle_commands(
    tcp_stream: &mut TcpListenerStream,
    tasks: &mut FuturesUnordered<JoinHandle<Result<TlsStream>>>,
) -> Result<TlsHandleCommands> {
    let ret = if tasks.is_empty() {
        match tcp_stream.try_next().await? {
            None => TlsHandleCommands::Break,
            Some(tcp_stream) => TlsHandleCommands::TcpStream(tcp_stream),
        }
    } else {
        select! {
            tcp_stream = tcp_stream.try_next() => {
                tcp_stream?.map_or(TlsHandleCommands::Break, TlsHandleCommands::TcpStream)
            }
            tls_stream = tasks.next() => {
                #[allow(clippy::expect_used)]
                let tls_stream = tls_stream.expect("FuturesUnordered stream can't be closed in ordinary circumstances")??;
                TlsHandleCommands::TlsStream(tls_stream)
            }
        }
    };

    Ok(ret)
}

#[cfg(feature = "use_native_tls")]
mod reexport {
    use crate::error::Result;
    use tokio_native_tls::native_tls::{self, Identity};
    use tracing::info;

    pub(crate) type TlsStream = tokio_native_tls::TlsStream<tokio::net::TcpStream>;
    pub(super) type TlsAcceptor = tokio_native_tls::TlsAcceptor;

    pub(super) fn create_acceptor(pem: &[u8], key: &[u8]) -> Result<TlsAcceptor> {
        info!("Use native-tls");

        let identity = Identity::from_pkcs8(pem, key)?;
        let acceptor = native_tls::TlsAcceptor::new(identity)?;

        Ok(acceptor.into())
    }
}

#[cfg(feature = "use_rustls")]
mod reexport {
    use crate::error::{Error, Result};
    use rustls_pemfile::{certs, pkcs8_private_keys};
    use std::{io::BufReader, sync::Arc};
    use tokio_rustls::rustls::{Certificate, PrivateKey, ServerConfig};
    use tracing::info;

    // Box because of: https://rust-lang.github.io/rust-clippy/master/index.html#large_enum_variant
    pub(crate) type TlsStream = Box<tokio_rustls::server::TlsStream<tokio::net::TcpStream>>;
    pub(super) type TlsAcceptor = tokio_rustls::TlsAcceptor;

    pub(super) fn create_acceptor(pem: &[u8], key: &[u8]) -> Result<TlsAcceptor> {
        info!("Use rustls");

        fn extract_single_key(data: Vec<Vec<u8>>) -> Result<Vec<u8>> {
            let [data]: [Vec<u8>; 1] = data
                .try_into()
                .map_err(|_| Error::CustomError("expect one key".into()))?;

            Ok(data)
        }

        let certs = certs(&mut BufReader::new(pem))?
            .drain(..)
            .map(Certificate)
            .collect::<Vec<_>>();
        let key = pkcs8_private_keys(&mut BufReader::new(key))
            .map(extract_single_key)?
            .map(PrivateKey)?;
        let config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(certs, key)?;

        Ok(Arc::new(config).into())
    }
}
