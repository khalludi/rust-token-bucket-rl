//! Limit the rate at which requests are processed.

mod layer;
#[allow(clippy::module_inception)]
mod rate;
mod service;
mod rate_limit_error;
mod response_future;

pub use self::{layer::RateLimitLayer, rate::Rate, service::RateLimit, rate_limit_error::RateLimitError, response_future::ResponseFuture};