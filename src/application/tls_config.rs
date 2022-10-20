#[cfg(all(feature = "rustls", not(any(feature = "native-tls"))))]
pub(crate) mod tls {
    use crate::error::Result;
    use axum::extract::connect_info::Connected;
    use hyper::server::conn::AddrStream;
    use std::net::SocketAddr;
    use std::sync::Arc;
    use tokio_rustls::rustls::{Certificate, PrivateKey, ServerConfig};
    use tokio_rustls::server::TlsStream;

    pub type Acceptor = tokio_rustls::TlsAcceptor;

    pub(crate) fn build_acceptor(pem: Vec<u8>, key: Vec<u8>) -> Result<Acceptor> {
        let mut buf = std::io::BufReader::new(&*pem);
        let certs = rustls_pemfile::certs(&mut buf)
            .unwrap()
            .into_iter()
            .map(Certificate)
            .collect();

        let mut buf = std::io::BufReader::new(&*key);
        let key = rustls_pemfile::pkcs8_private_keys(&mut buf)
            .unwrap()
            .into_iter()
            .map(PrivateKey)
            .next()
            .unwrap();

        Ok(Arc::new(
            ServerConfig::builder()
                .with_safe_defaults()
                .with_no_client_auth()
                .with_single_cert(certs, key)?,
        )
        .into())
    }

    #[derive(Debug, Clone)]
    /// Wrapper for SocketAddr to implement [`Connected`] so
    /// we can run [`axum::routing::Router::into_make_service_with_connect_info`] with [`TlsStream<AddrStream>`]
    pub struct RemoteAddr(pub SocketAddr);

    impl Connected<&TlsStream<AddrStream>> for RemoteAddr {
        fn connect_info(target: &TlsStream<AddrStream>) -> Self {
            RemoteAddr(target.get_ref().0.remote_addr())
        }
    }
}

#[cfg(all(feature = "native-tls", not(any(feature = "rustls"))))]
pub(crate) mod tls {
    use crate::error::Result;
    use axum::extract::connect_info::Connected;
    use hyper::server::conn::AddrStream;
    use std::net::SocketAddr;
    use tokio_native_tls::native_tls::{Identity, TlsAcceptor};
    use tokio_native_tls::TlsStream;

    /// Alias for [`tokio_native_tls::TlsAcceptor`]
    pub type Acceptor = tokio_native_tls::TlsAcceptor;

    /// builds [`Acceptor`]
    pub(crate) fn build_acceptor(pem: Vec<u8>, key: Vec<u8>) -> Result<Acceptor> {
        let identity = Identity::from_pkcs8(&pem, &key)?;
        TlsAcceptor::builder(identity)
            .build()
            .map(From::from)
            .map_err(Into::into)
    }

    #[derive(Debug, Clone)]
    /// Wrapper for SocketAddr to implement [`Connected`] so
    /// we can run [`axum::routing::Router::into_make_service_with_connect_info`] with [`TlsStream<AddrStream>`]
    pub struct RemoteAddr(pub SocketAddr);

    impl Connected<&TlsStream<AddrStream>> for RemoteAddr {
        fn connect_info(target: &TlsStream<AddrStream>) -> Self {
            RemoteAddr(target.get_ref().get_ref().get_ref().remote_addr())
        }
    }
}
