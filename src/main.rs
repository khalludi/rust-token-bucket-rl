pub mod rate_limit;

use crate::rate_limit::RateLimitError;

pub use self::{
    rate_limit::{RateLimit, RateLimitLayer},
};

use axum::{
    response::{IntoResponse},
    routing::{get}, Router, error_handling::HandleErrorLayer, http::{StatusCode, Method},
    BoxError, Json,
};
use rand::Rng;
use tower::{
    ServiceBuilder
};

use tower_http::{trace::TraceLayer, cors::{CorsLayer, Any}};
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

    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        // allow requests from any origin
        .allow_origin(Any);

    // Compose the routes
    let app = Router::new()
        .route("/", get(hello_world))
        // Add middleware to all routes
        .layer(ServiceBuilder::new()
            .layer(cors)
        )
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
                .layer(RateLimitLayer::new(5, Duration::from_secs(15)))
                .layer(TraceLayer::new_for_http())
        );
    let addr = SocketAddr::from(([127, 0, 0, 1], 4000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn hello_world() -> impl IntoResponse {
    let adjectives = ["adorable","amusing", "awesome", 
        "bright", "beautiful", "calm", "excited", "fantastic", 
        "friendly", "good", "happy", "sensible", "spicy", "sturdy", 
        "truthful", "wonderful"];

    let mut rng = rand::thread_rng();
    let n1: usize = rng.gen_range(0..adjectives.len());


    Json(adjectives[n1].to_string())
}