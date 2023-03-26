use hyper::{
    rt::Executor,
    server::Builder,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use std::convert::Infallible;
use std::net::SocketAddr;
use tracing::{error, info};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use treadmill::Runtime;
use treadmill_hyper::TreadmillExecutor;

async fn handle(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from("Hello World")))
}

fn main() {
    setup_logging();
    // Construct our SocketAddr to listen on...
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    let rt = Runtime::default();
    rt.block_on(async {
        // And a MakeService to handle each connection...
        //let make_service =
        //    make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle)) });

        // So I need to provide an acceptor which listens to TCP not implemented in tokio to be
        // able to provide my own runtime. TODO
        //let server = Server::bind(&addr)
        //    .executor(TreadmillExecutor)
        //    .serve(make_service);

        //if let Err(e) = server.await {
        //    error!("Server error: {}", e);
        //}
    });
}

fn setup_logging() {
    let env_filter = match EnvFilter::try_from_default_env() {
        Ok(filter) => filter,
        Err(_) => EnvFilter::new("treadmill=trace,server=info"),
    };

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(env_filter)
        .init();
}
