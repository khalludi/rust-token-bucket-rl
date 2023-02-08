use std::{task::{Poll, Context}, pin::Pin};

use axum::BoxError;
use futures_core::Future;
use pin_project::pin_project;
use tokio::time::Sleep;

#[pin_project]
pub struct ResponseFuture<F> {
    #[pin]
    pub response_future: F,
}

impl<F, Response, Error> Future for ResponseFuture<F>
where
    F: Future<Output = Result<Response, Error>>,
    Error: Into<BoxError>,
{
    type Output = Result<Response, BoxError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        println!("Poll in Response Future 1");

        match this.response_future.poll(cx) {
            Poll::Ready(result) => {
                println!("Poll in Response Future 2");
                let result = result.map_err(Into::into);
                return Poll::Ready(result);
            }
            Poll::Pending => {}
        }

        println!("Poll in Response Future 3");

        Poll::Pending

        // match this.sleep.poll(cx) {
        //     Poll::Ready(()) => {
        //         let error = Box::new(TimeoutError(()));
        //         return Poll::Ready(Err(error));
        //     }
        //     Poll::Pending => {}
        // }

        // Poll::Pending
    }
}