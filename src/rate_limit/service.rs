use super::{Rate, ResponseFuture};
use crate::{BoxError, rate_limit::RateLimitError};
use futures_core::ready;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::time::{Instant, Sleep};
use tower_service::Service;

/// Enforces a rate limit on the number of requests the underlying
/// service can handle over a period of time.
#[derive(Debug)]
pub struct RateLimit<T> {
    inner: T,
    rate: Rate,
    state: State,
    sleep: Pin<Box<Sleep>>,
}

#[derive(Debug, Clone)]
enum State {
    // The service has hit its limit
    Limited,
    Ready { until: Instant, rem: u64 },
}

impl<T> RateLimit<T> {
    /// Create a new rate limiter
    pub fn new(inner: T, rate: Rate) -> Self {
        let until = Instant::now();
        let state = State::Ready {
            until,
            rem: rate.num(),
        };

        RateLimit {
            inner,
            rate,
            state,
            // The sleep won't actually be used with this duration, but
            // we create it eagerly so that we can reset it in place rather than
            // `Box::pin`ning a new `Sleep` every time we need one.
            sleep: Box::pin(tokio::time::sleep_until(until)),
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
        println!("Rate Limit Service #1");
        match self.state {
            State::Ready { .. } => {
                println!("Rate Limit Service #2");
                return Poll::Ready(ready!(self.inner.poll_ready(cx).map_err(Into::into)))
            },
            State::Limited => {
                if Pin::new(&mut self.sleep).poll(cx).is_pending() {
                    println!("Inside limited");

                    tracing::trace!("rate limit exceeded; sleeping.");
                    return Poll::Ready(Err(Box::new(RateLimitError(()))));
                    // return Poll::Pending;
                }
            }
        }
        println!("Rate Limit Service #3");
        self.state = State::Ready {
            until: Instant::now() + self.rate.per(),
            rem: self.rate.num(),
        };
        println!("Rate Limit Service #4");
        Poll::Ready(ready!(self.inner.poll_ready(cx).map_err(Into::into)))
    }

    fn call(&mut self, request: Request) -> Self::Future {
        match self.state {
            State::Ready { mut until, mut rem } => {
                let now = Instant::now();

                println!("Now: {:?}", now);
                println!("Until: {:?}", until);
                println!("Rem: {}", rem);

                // If the period has elapsed, reset it.
                if now >= until {
                    until = now + self.rate.per();
                    rem = self.rate.num();
                }
                println!("New Until: {:?}", until);

                if rem > 1 {
                    rem -= 1;
                    self.state = State::Ready { until, rem };
                } else {
                    // The service is disabled until further notice
                    // Reset the sleep future in place, so that we don't have to
                    // deallocate the existing box and allocate a new one.
                    self.sleep.as_mut().reset(until);
                    self.state = State::Limited;
                }

                // Call the inner future
                ResponseFuture {
                    response_future: self.inner.call(request),
                }   
            }
            State::Limited => panic!("service not ready; poll_ready must be called first"),
        }
    }
}