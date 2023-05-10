//! In theory it's very easy to get your own asynchronous executor to drive hyper. So lets see if
//! that's the case...
//!
//! Reference https://github.com/async-rs/async-std-hyper
use async_io::Async;
use futures_lite::io::{AsyncRead, AsyncWrite};
use futures_lite::StreamExt;
use hyper::rt::Executor;
use std::future::Future;
use std::io;
use std::net::{TcpListener, TcpStream};
use std::pin::Pin;
use std::task::{Context, Poll};
use tracing::trace;

#[derive(Clone, Copy)]
pub struct TreadmillExecutor;

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

pub struct TreadmillListener {
    io: Async<TcpListener>,
}

impl TreadmillListener {
    pub fn new(io: TcpListener) -> io::Result<Self> {
        let io = Async::new(io)?;
        Ok(Self { io })
    }
}

#[cfg(feature = "server")]
impl hyper::server::accept::Accept for TreadmillListener {
    type Conn = TreadmillStream;
    type Error = io::Error; // Work out this

    fn poll_accept(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
        trace!("called accept");
        if let Poll::Ready(res) = Box::pin(self.io.incoming()).poll_next(cx) {
            trace!("Accepted connection");
            if let Some(stream) = res {
                trace!("Some?");
                Poll::Ready(Some(stream.map(|stream| TreadmillStream { stream })))
            } else {
                trace!("None");
                Poll::Ready(None)
            }
        } else {
            trace!("Pending");
            Poll::Pending
        }
    }
}

pub struct TreadmillStream {
    stream: Async<TcpStream>,
}

impl tokio::io::AsyncRead for TreadmillStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut tokio::io::ReadBuf,
    ) -> Poll<io::Result<()>> {
        if let Poll::Ready(bytes) =
            // TODO initialize_unfilled is gonna suck
            Pin::new(&mut self.stream).poll_read(cx, buf.initialize_unfilled())?
        {
            buf.set_filled(bytes);
            trace!("Read {} bytes from a TcpStream", bytes);
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    }
}

impl tokio::io::AsyncWrite for TreadmillStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.stream).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        Pin::new(&mut self.stream).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        Pin::new(&mut self.stream).poll_close(cx)
    }
}
