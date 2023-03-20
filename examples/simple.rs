use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use futures_lite::future;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use treadmill::Runtime;

struct SayNTimes {
    to_say: String,
    n: usize,
    done: usize,
}

impl SayNTimes {
    fn new(to_say: String, n: usize) -> Self {
        Self { to_say, n, done: 0 }
    }
}

impl Future for SayNTimes {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<()> {
        println!("{}", self.to_say);
        self.done += 1;
        if self.done == self.n {
            Poll::Ready(())
        } else {
            ctx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

fn main() {
    setup_logging();

    let rt = Runtime::default();
    let fut = SayNTimes::new("Hello world".to_string(), 5);
    let task = rt.spawn(fut);

    rt.block_on(task);
}

fn setup_logging() {
    let env_filter = match EnvFilter::try_from_default_env() {
        Ok(filter) => filter,
        Err(_) => EnvFilter::new("treadmill=trace,simple=info"),
    };

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(env_filter)
        .init();
}
