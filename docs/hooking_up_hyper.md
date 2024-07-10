# Hooking up Treadmill to Hyper

It's only natural once you have a working asynchronous runtime you want to
start creating more useful applications with it and not just toy programs
to demonstrate the basic functionality. And I'm sure like most people with
cause to use async most of my async usage is creating web APIs.

From reading about some of the work to integrate hyper into curl I was aware
that hyper is runtime agnostic, therefore in theory I should be able to just
implement a single trait and hyper can use treadmill to execute tasks.

This is going to detail my efforts to do this using hyper 0.14.x.

## Implementing the Executor Trait

If we go to the hyper runtime module we see there's a single trait to implement
[`Executor`](https://docs.rs/hyper/latest/hyper/rt/trait.Executor.html). Which
looks like follows:

```rust
pub trait Executor<Fut> {
    // Required method
    fn execute(&self, fut: Fut);
}
```

This seems easy, we just have to provide means to submit a future to the
runtime. So for an initial implementation I just did:

```rust
impl<F> Executor<F> for TreadmillExecutor
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    fn execute(&self, fut: F) {
        trace!("Executing future for hyper");
        treadmill::spawn(fut).detach();
    }
}
```

Right that was simple, now just to create a server and a client and give myself
a pat on the back.

So here's the code for a simple server example:

```rust
async fn handle(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from("Hello World")))
}

#[treadmill::main]
async fn main() {
    // Construct our SocketAddr to listen on...
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    // And a MakeService to handle each connection...
    let make_service =
        make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle)) });

    let server = Server::bind(&addr)
        .executor(TreadmillExecutor)
        .serve(make_service);

    if let Err(e) = server.await {
        error!("Server error: {}", e);
    }
}

```

Now this didn't compile as it stood, it was missing the tcp feature. So I
enabled that feature thinking it may use mio internally as hyper isn't tied
to any specific runtime.

Well it turns out it uses `tokio::net` for TCP socket types. Time to begin the
next challenge and figure out how to add our own TCP types. 

## Implementing the Server Gubbins

So the type we need will probably be an option in the builder or the
server itself so first off I'll look to the `Server` docs. 

Server is defined as:

```rust
pub struct Server<I, S, E = Exec> { /* private fields */ }
```

And for `Server::bind` the `I` generic is returned as `AddressIncoming` which
implemented the
[`Accept` trait](https://docs.rs/hyper/latest/hyper/server/accept/trait.Accept.html) 
in hyper. However, it doesn't have any generic bounds applied until `server::Builder::serve`
which does apply an `Accept` trait bound.

Peeking at the `AddrIncoming` source I can see the struct contains a `TcpListener`,
and it was this point I figured I should do a tokio and have a mio based IO driver
in my runtime. And while I may do this in future I had some troubles initially.


I actually spent a moderate amount of time trying to work out how mio worked and
implenting my own versions of the Tcp types. But I couldn't see a way to get this
working without storing the wakers somewhere and for simplicitly I'm currently 
using async-task so I don't have my wakers available. And while I may gradually
rip out a lot of these util crates as I explore lower level details I didn't
want to do this initially.

So I'll skip that month of on-off work I threw away in favour of using 
[`async_io`](https://crates.io/crates/async-io). I also found a reference on 
writing a hyper executor which looked sane done by the async-std folks 
[here](https://crates.io/crates/async-io).

So with the following Listener I eagerly went to run my server and start processing
requests:

```rust

pub struct TreadmillListener {
    io: Async<TcpListener>,
}

impl hyper::server::accept::Accept for HyperListener {
    type Conn = HyperStream;
    type Error = io::Error;

    fn poll_accept(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {A
        match Box::pin(self.io.incoming()).poll_next(cx) {
            Poll::Ready(res) => Poll::Ready(res.map(|x| x.map(|stream| TreadmillStream { stream }))),
            Poll::Pending => Poll::Pending
        }
    }
}

```

And it compiled and ran. But it hung never accepting any requests. So what gives?

Emitting a tracing event every time `poll_accept` is called immediately shines
some light on what's happening. The poll function is only called once. A future
is only polled when it's woken up after the first poll. However, when
`poll_accept` is called at the end it drops the incoming `Stream` and that task
is cancelled and never woken up again.

Here is an area where having a separate IO driver and moving IO tasks to
another thread would allow the above code to work. But that's not what I've
been doing so how do I progress?

Initially I attempted to store the stream to keep it alive or move it to
another thread or task, but hairy lifetime errors kept slapping me down.
Unfortunately, none of these failed attempts were committed so I can't demonstrate
any with minimal effort so just take my word for it.

But hyper has some utility functions to create an `Accept` implementation from
a stream or a polling function. So here I tried to do it using `server::accept::poll_fn`
as shown below:

```rust
use hyper::server::accept;

impl TreadmillListener {
    pub async fn accept(&self) -> Option<io::Result<TreadmillStream>> {
        let mut incoming = pin!(self.io.incoming());
        let stream = incoming.next().await?;
        Some(stream.map(|stream| TreadmillStream { stream }))
    }
}

let connection_acceptor = accept::poll_fn(self.accept());
```

And now my example ran, and it read data! However...

```
hread 'main' panicked at '`async fn` resumed after completion', /home/daniel/personal/treadmill/treadmill-hyper/src/lib.rs:42:71
```

Something caused this future to be polled after it had completed. I tried to
figure this out but didn't really get anywhere and didn't particularly want to
dive into the hyper source code to figure it out at this moment so I just
fell back to the stream:

```rust

use hyper::server::accept;

impl TreadmillListener {
    pub fn accept_stream(&self) -> impl Stream<Item = io::Result<TreadmillStream>> + '_ {
        self.io
            .incoming()
            .map(|x| x.map(|stream| TreadmillStream { stream }))
    }

    pub fn request_acceptor(&self) -> impl Accept<Conn = TreadmillStream, Error = io::Error> + '_ {
        accept::from_stream(self.accept_stream())
    }
}
```

And this works, we now have a working server using treadmill as the runtime!
Now onto the client, that can't be too hard.

## Doing the client (it's always DNS)

Now older and wiser I knew there would likely be an equivalent to `Accept` in
the client or it's builder struct that I'd have to implement some stuff for.

And there is, in the `client::connect` module there's the `Connect` trait, but
this trait is sealed. So how do we go about implementing it? Well there's an
implementation for a generic type `S` defined as follows:

```rust
impl<S, T> Connect for S
where
    S: Service<Uri, Response = T> + Send + 'static,
    S::Error: Into<Box<dyn StdError + Send + Sync>>,
    S::Future: Unpin + Send,
    T: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
```

Okay so time to implement `Service` for a `TreadmillConnector` type and then
eventually we should see the following code work:

```rust
let client: Client<TreadmillConnector, hyper::Body> = Client::builder()
    .executor(TreadmillExecutor)
    .build(TreadmillConnector);
```

DNS DNS DNS

```rust
#[derive(Clone)]
pub struct TreadmillConnector;

impl TreadmillConnector {
    async fn call(&self, uri: Uri) -> io::Result<TreadmillStream> {
        let port = match uri.port_u16() {
            Some(p) => p,
            None => 80, // TODO do the correct default port for the protocol
        };
        let ip = treadmill::spawn_blocking(move || resolve_ip(uri))
            .await
            .unwrap();

        let stream = TcpStream::connect((ip, port))?;
        Ok(TreadmillStream::new(stream)?)
    }
}

fn resolve_ip(uri: Uri) -> IpAddr {
    let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default()).unwrap();
    let response = resolver.lookup_ip(uri.host().unwrap().to_string()).unwrap();
    response.iter().next().expect("no addresses returned!")
}

impl Service<Uri> for TreadmillConnector {
    type Response = TreadmillStream;
    type Error = io::Error;

    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, uri: Uri) -> Self::Future {
        let this = self.clone();
        Box::pin(async move { this.call(uri).await })
    }
}
```

So things I had to implement for this:

1. `spawn_blocking`
2. DNS lookup via trust-dns (which is using tokio)
3. Something to make the runtime findable on threads spawned (ahhh `thread_local` induced pain)
