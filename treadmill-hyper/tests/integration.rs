use hyper::{
    client::Client,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use std::convert::Infallible;
use std::net::TcpListener;
use treadmill::Runtime;
use treadmill_hyper::{TreadmillConnector, TreadmillExecutor, TreadmillListener};

async fn echo(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(req.into_body()))
}

#[treadmill::test]
async fn echo_server() {
    // And a MakeService to handle each connection...
    let make_service = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(echo)) });

    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
    let listener = TreadmillListener::new(listener).unwrap();

    let server = Server::builder(listener.request_acceptor())
        .executor(TreadmillExecutor)
        .serve(make_service);

    let client: Client<TreadmillConnector, hyper::Body> = Client::builder()
        .executor(TreadmillExecutor)
        .build(TreadmillConnector);
}
