use crate::*;
use hyper::client::connect::{Connected, Connection};
use hyper::service::Service;
use hyper::Uri;
use std::{
    future::Future,
    pin::Pin,
    task::{self, Poll},
};

impl Connection for TreadmillStream {
    fn connected(&self) -> Connected {
        // So I guess we need to fill in the metadata somehow
        Connected::new()
    }
}

#[derive(Clone)]
pub struct TreadmillConnector;

impl Service<Uri> for TreadmillConnector {
    type Response = TreadmillStream;
    type Error = io::Error;

    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: Uri) -> Self::Future {
        // We need to do a DNS resolution and
        todo!();
    }
}
