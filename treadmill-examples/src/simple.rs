use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
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
            // When we call await on a future it will be scheduled once by the executor. After that
            // it's waker (which is accessible from the context) needs to wake it up to schedule it
            // again. This avoids us calling poll when the future can't progress and busy-looping.
            // As this is a very simple future we just call `Waker::wake_by_ref` each call because
            // we know it will be ready to progress next poll for sure!
            ctx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

#[treadmill::main]
async fn main() {
    setup_logging();

    let fut = SayNTimes::new("Hello world".to_string(), 5);
    treadmill::spawn(fut).await
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
