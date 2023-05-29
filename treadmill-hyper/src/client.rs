use crate::*;
use hyper::client::connect::{Connected, Connection};
use hyper::service::Service;
use hyper::Uri;
use std::net::*;
use std::{
    future::Future,
    pin::Pin,
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

fn resolve_ip(uri: Uri) -> IpAddr {
    let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default()).unwrap();

    // Lookup the IP addresses associated with a name.
    // The final dot forces this to be an FQDN, otherwise the search rules as specified
    //  in `ResolverOpts` will take effect. FQDN's are generally cheaper queries.
    let response = resolver.lookup_ip("www.example.com.").unwrap();

    response.iter().next().expect("no addresses returned!")
}
