//! In theory it's very easy to get your own asynchronous executor to drive hyper. So lets see if
//! that's the case...
//!
//! Reference https://github.com/async-rs/async-std-hyper
use hyper::rt::Executor;
use std::future::Future;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Clone, Copy)]
pub struct TreadmillExecutor;

impl<F> Executor<F> for TreadmillExecutor
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    fn execute(&self, fut: F) {
        treadmill::spawn(fut).detach();
    }
}

pub struct TreadmillListener;

#[cfg(feature = "server")]
impl hyper::server::accept::Accept for TreadmillListener {
    type Conn = TreadmillStream;
    type Error = (); // Work out this

    fn poll_accept(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
        todo!()
    }
}

pub struct TreadmillStream;

impl tokio::io::AsyncRead for TreadmillStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut tokio::io::ReadBuf,
    ) -> Poll<io::Result<()>> {
        todo!()
    }
}

impl tokio::io::AsyncWrite for TreadmillStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        todo!()
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        todo!()
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        todo!()
    }
}
