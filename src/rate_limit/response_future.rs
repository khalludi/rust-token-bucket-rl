use std::{task::{Poll, Context}, pin::Pin};

use axum::BoxError;
use futures_core::{Future, ready};
use pin_project::pin_project;
use tokio::{sync::OwnedSemaphorePermit};

#[pin_project]
pub struct ResponseFuture<F> {
    #[pin]
    pub response_future: F,
    pub _permit: OwnedSemaphorePermit,
}

impl<F, Response, Error> Future for ResponseFuture<F>
where
    F: Future<Output = Result<Response, Error>>,
    Error: Into<BoxError>,
{
    type Output = Result<Response, BoxError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(ready!(self.project().response_future.poll(cx).map_err(Into::into)))
    }
}