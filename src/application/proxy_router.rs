//! Tool to implement reverse-proxy logic in axum::Router

use axum::{
    extract::Extension,
    http::StatusCode,
    http::{uri::Uri, Request},
    response::IntoResponse,
    routing::any,
    Router,
};
use hyper::{client::HttpConnector, Body};

type Client = hyper::client::Client<HttpConnector, Body>;

// TODO: might need to be removed, review it on axum 0.6
// TODO: remove allow
#[allow(clippy::expect_used)]
async fn proxy_handler(
    Extension(client): Extension<Client>,
    Extension(destination): Extension<String>,
    mut request: Request<Body>,
) -> impl IntoResponse {
    let path_query = request
        .uri()
        .path_and_query()
        .map(|v| v.as_str())
        .unwrap_or_else(|| request.uri().path());

    let uri = format!("{destination}{path_query}");
    //TODO: return error in response
    *request.uri_mut() = Uri::try_from(uri).expect("Failed to get uri from destination");

    let response = client.request(request).await;
    match response {
        Ok(resp) => resp.into_response(),
        // FIXME(kos): This may leak private information to the client.
        //             Be careful of what gets sent in `err`.
        //
        //             Alternatively you can do so only in "debug mode" (or at least a
        //             debug build) of the application, and return some generic
        //             error message to the client in production build, while
        //             always writing to the logs the actual full error.
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.message().to_string()).into_response(),
    }
}

/// Returns [`Router`] with set handler which proxy all incoming requests to destination
pub fn route_proxy(path: &str, destination: &str) -> Router {
    let client = Client::new();

    Router::new()
        .route(path, any(proxy_handler))
        .layer(Extension(client))
        .layer(Extension(destination.to_owned()))
}
