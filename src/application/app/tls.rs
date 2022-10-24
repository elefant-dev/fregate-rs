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
use std::{future::ready, net::SocketAddr, sync::Arc};
use tokio::{
    net::{TcpListener, TcpStream},
    select,
};
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
    let mut tcp_stream = TcpListenerStream::new(listener);

    // TODO: https://github.com/tokio-rs/async-stream/issues/81
    let acceptor = Arc::new(acceptor);
    let ret = stream! {
        let mut tasks = FuturesUnordered::new();

        loop {
            if tasks.is_empty() {
                match tcp_stream.try_next().await {
                    Ok(None) => break,
                    Ok(Some(stream)) => {
                        let acceptor = acceptor.clone();
                        tasks.push(tokio::task::spawn(async move { Ok::<_, Error>(acceptor.accept(stream).await?) }));
                    },
                    Err(error) => yield Err(Error::from(error)),
                }
                continue
            }

            select! {
                stream = tcp_stream.try_next() => {
                    match stream {
                        Ok(None) => break,
                        Ok(Some(stream)) => {
                            let acceptor = acceptor.clone();
                            tasks.push(tokio::task::spawn(async move { Ok::<_, Error>(acceptor.accept(stream).await?) }));
                        },
                        Err(error) => yield Err(Error::from(error)),
                    }
                }
                accept = tasks.next() => {
                    match accept.expect("FuturesUnordered stream can't be closed in ordinary circumstances") {
                        Ok(Ok(tls_stream)) => yield Ok(tls_stream),
                        Ok(Err(error)) => yield Err(Error::from(error)),
                        Err(error) => yield Err(Error::from(error)),
                    }
                }
            }
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
