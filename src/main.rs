pub mod rate_limit;

pub use self::{
    rate_limit::{RateLimit, RateLimitLayer},
};

use axum::{
    response::IntoResponse,
    routing::{get}, Router,
};
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
        .layer(RateLimitLayer::new(5, Duration::new(60, 0)));

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