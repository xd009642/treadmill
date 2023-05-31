use crate::*;
use hyper::client::connect::{Connected, Connection};
use hyper::service::Service;
use hyper::Uri;
use std::net::*;
use std::{
    future::Future,
    pin::{pin, Pin},
    task::{self, Poll},
};
use trust_dns_resolver::config::*;
use trust_dns_resolver::Resolver;

impl Connection for TreadmillStream {
    fn connected(&self) -> Connected {
        // So I guess we need to fill in the metadata somehow
        Connected::new()
    }
}

#[derive(Clone)]
pub struct TreadmillConnector;

impl TreadmillConnector {
    async fn call(&self, uri: Uri) -> io::Result<TreadmillStream> {
        let port = match uri.port_u16() {
            Some(p) => p,
            None => 80, // TODO
        };
        let ip = treadmill::spawn_blocking(move || resolve_ip(uri))
            .await
            .unwrap();

        let listener = TcpListener::bind((ip, port))?;
        pin!(TreadmillListener::new(listener)?.accept_stream())
            .next()
            .await
            .unwrap()
    }
}

impl Service<Uri> for TreadmillConnector {
    type Response = TreadmillStream;
    type Error = io::Error;

    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        // I don't think I need to do anything here because the DNS is running in a separate tokio
        // runtime so I can't really pass the context in
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, uri: Uri) -> Self::Future {
        let this = self.clone();
        Box::pin(async move { this.call(uri).await })
    }
}

fn resolve_ip(uri: Uri) -> IpAddr {
    let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default()).unwrap();

    let response = resolver.lookup_ip(uri.to_string()).unwrap();

    response.iter().next().expect("no addresses returned!")
}
