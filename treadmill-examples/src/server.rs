use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use std::convert::Infallible;
use std::net::TcpListener;
use tracing::{error, info};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use treadmill::Runtime;
use treadmill_hyper::{TreadmillExecutor, TreadmillListener};

async fn handle(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    info!("Got a request!");
    Ok(Response::new(Body::from("Hello World")))
}

fn main() {
    setup_logging();

    let rt = Runtime::default();
    rt.block_on(async {
        // And a MakeService to handle each connection...
        let make_service =
            make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle)) });

        let listener = TcpListener::bind("0.0.0.0:8080").unwrap();

        let listener = TreadmillListener::new(listener).unwrap();

        let server = Server::builder(listener.request_acceptor())
            .executor(TreadmillExecutor)
            .serve(make_service);

        if let Err(e) = server.await {
            error!("Server error: {}", e);
        }
    });
}

fn setup_logging() {
    let env_filter = match EnvFilter::try_from_default_env() {
        Ok(filter) => filter,
        Err(_) => EnvFilter::new("treadmill=trace,server=info,hyper=trace"),
    };

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(env_filter)
        .init();
}
