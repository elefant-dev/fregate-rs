use crate::tls::TlsStream;
use axum::extract::connect_info::Connected;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// Wrapper for SocketAddr to implement [`Connected`] so
/// we can run [`axum::routing::Router::into_make_service_with_connect_info`] with [`TlsStream<AddrStream>`]
pub struct RemoteAddr(pub SocketAddr);

#[cfg(feature = "native-tls")]
impl Connected<&TlsStream> for RemoteAddr {
    fn connect_info(target: &TlsStream) -> Self {
        Self(
            target
                .get_ref()
                .get_ref()
                .get_ref()
                .peer_addr()
                .unwrap_or(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0)),
        )
    }
}

#[cfg(feature = "rustls")]
impl Connected<&TlsStream> for RemoteAddr {
    fn connect_info(target: &TlsStream) -> Self {
        Self(
            target
                .get_ref()
                .0
                .peer_addr()
                .unwrap_or(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0)),
        )
    }
}
