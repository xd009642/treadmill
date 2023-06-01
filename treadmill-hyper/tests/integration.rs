use async_channel::bounded;
use hyper::{
    body::Buf,
    client::Client,
    service::{make_service_fn, service_fn},
    Body, Method, Request, Response, Server, Uri,
};
use std::convert::Infallible;
use std::net::TcpListener;
use tracing::info;
use tracing_test::traced_test;
use treadmill::Runtime;
use treadmill_hyper::{TreadmillConnector, TreadmillExecutor, TreadmillListener};

async fn echo(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(req.into_body()))
}

#[treadmill::test]
#[traced_test]
async fn echo_server() {
    let (tx, rx) = bounded(1);

    // And a MakeService to handle each connection...
    let make_service = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(echo)) });

    let listener = TcpListener::bind("0.0.0.0:0").unwrap();
    let address = format!("http://localhost:{}", listener.local_addr().unwrap().port())
        .parse::<Uri>()
        .unwrap();

    info!("Address: {}", address);

    let listener = TreadmillListener::new(listener).unwrap();

    treadmill::spawn(async move {
        let server = Server::builder(listener.request_acceptor())
            .executor(TreadmillExecutor)
            .serve(make_service)
            .with_graceful_shutdown(async {
                let _ = rx.recv().await;
            });
        if let Err(e) = server.await {
            println!("Server error: {}", e);
        }
    })
    .detach();

    let client: Client<TreadmillConnector, hyper::Body> = Client::builder()
        .executor(TreadmillExecutor)
        .build(TreadmillConnector);

    let req = Request::builder()
        .method(Method::POST)
        .uri(address)
        .body(Body::from("Hello world"))
        .expect("Request building failed");

    let response = client.request(req).await.unwrap();

    let body = hyper::body::to_bytes(response).await.unwrap();

    assert_eq!(body.chunk(), b"Hello world");

    let _ = tx.send(()).await; // I don't care too much, but lets try to clean up things
}
