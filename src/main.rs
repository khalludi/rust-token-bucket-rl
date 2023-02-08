pub mod rate_limit;

use crate::rate_limit::RateLimitError;

pub use self::{
    rate_limit::{RateLimit, RateLimitLayer},
};

use axum::{
    response::IntoResponse,
    routing::{get}, Router, error_handling::HandleErrorLayer, http::StatusCode,
    BoxError,
};
use tower::{
    ServiceBuilder
};
use tower::buffer::BufferLayer;
use tower_http::trace::TraceLayer;
use std::{
    net::SocketAddr,
    time::Duration,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_todos=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Compose the routes
    let app = Router::new()
        .route("/", get(hello_world))
        // Add middleware to all routes
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|err: BoxError| async move {
                    if err.is::<RateLimitError>() {
                        (
                            StatusCode::TOO_MANY_REQUESTS,
                            format!("Too many requests: {}", err),
                        )
                    } else {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Unhandled error: {}", err),
                        )
                    }
                }))
                .layer(BufferLayer::new(1024))
                .layer(RateLimitLayer::new(5, Duration::from_secs(15)))
                .layer(TraceLayer::new_for_http())
        );
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn hello_world() -> impl IntoResponse {
    "Hello, World!"
}

async fn handle_timeout_error(err: BoxError) -> (StatusCode, String) {
    if err.is::<tower::timeout::error::Elapsed>() {
        (
            StatusCode::REQUEST_TIMEOUT,
            "Request took too long".to_string(),
        )
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Unhandled internal error: {}", err),
        )
    }
}