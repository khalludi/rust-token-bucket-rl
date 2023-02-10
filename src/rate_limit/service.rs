use super::{Rate, ResponseFuture};
use crate::{BoxError, rate_limit::RateLimitError};
use futures_core::ready;
use std::{
    task::{Context, Poll}, sync::{Arc, Mutex}, time::Duration,
};
use tokio::{time::Instant, sync::{OwnedSemaphorePermit, Semaphore}};
use tokio_util::sync::PollSemaphore;
use tower_service::Service;

/// Enforces a rate limit on the number of requests the underlying
/// service can handle over a period of time.
#[derive(Debug)]
pub struct RateLimit<T> {
    inner: T,
    rate: Rate,
    last_token_refresh: Arc<Mutex<Instant>>,
    tokens: Arc<Mutex<usize>>,
    permit_semaphore: PollSemaphore,
    permit: Option<OwnedSemaphorePermit>,
}

impl<T> RateLimit<T> {
    /// Create a new rate limiter
    pub fn new(inner: T, rate: Rate) -> Self {
        let rate_num: usize = rate.num().try_into().unwrap();

        RateLimit {
            inner,
            rate,
            last_token_refresh: Arc::new(Mutex::new(Instant::now())),
            tokens: Arc::new(Mutex::new(rate_num)),
            permit_semaphore: PollSemaphore::new(Arc::new(Semaphore::new(rate_num))),
            permit: None,
        }
    }

    /// Get a reference to the inner service
    pub fn get_ref(&self) -> &T {
        &self.inner
    }

    /// Get a mutable reference to the inner service
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Consume `self`, returning the inner service
    pub fn into_inner(self) -> T {
        self.inner
    }

    fn refresh_tokens(&mut self) {
        let mut tokens_lock = self.tokens.lock().unwrap();
        let mut token_refresh_lock = self.last_token_refresh.lock().unwrap();
        let elapsed = token_refresh_lock.elapsed();
        
        if elapsed > self.rate.per() {
            let remainder_time: u64 = elapsed.as_secs() % self.rate.per().as_secs();
            *tokens_lock = self.rate.num();
            *token_refresh_lock = Instant::now();
            match (*token_refresh_lock).checked_sub(Duration::from_secs(remainder_time)) {
                Some(value) => {*token_refresh_lock = value;}
                None => {println!("failed")}
            }
        }
    }
}

impl<S, Request> Service<Request> for RateLimit<S>
where
    S: Service<Request>,
    S::Error: Into<BoxError>,
{
    type Response = S::Response;
    type Error = BoxError;
    type Future = ResponseFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), BoxError>> {
        if self.permit.is_none() {
            // Try refreshing tokens
            self.refresh_tokens();
            
            let mut tokens_lock = self.tokens.lock().unwrap();
            if *tokens_lock > 0 {
                self.permit = ready!(self.permit_semaphore.poll_acquire(cx));
                *tokens_lock -= 1;
                debug_assert!(
                    self.permit.is_some(),
                    "RateLimit semaphore is never closed, so `poll_acquire` \
                    should never fail",
                );
            } else {
                println!("Poll_ready, no tokens");

                return Poll::Ready(Err(Box::new(RateLimitError(()))));
            }
        }

        // Once we've acquired a permit (or if we already had one), poll the
        // inner service.
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        // Take the permit
        let permit = self
            .permit
            .take()
            .expect("max requests in-flight; poll_ready must be called first");

        // Call the inner service
        let future = self.inner.call(request);

        ResponseFuture {
            response_future: future,
            _permit: permit,
        }
    }
}

impl<T: Clone> Clone for RateLimit<T> {
    fn clone(&self) -> Self {
        // Since we hold an `OwnedSemaphorePermit`, we can't derive `Clone`.
        // Instead, when cloning the service, create a new service with the
        // same semaphore, but with the permit in the un-acquired state.
        Self {
            inner: self.inner.clone(),
            rate: self.rate.clone(),
            last_token_refresh: self.last_token_refresh.clone(),
            tokens: self.tokens.clone(),
            permit_semaphore: self.permit_semaphore.clone(),
            permit: None,
        }
    }
}