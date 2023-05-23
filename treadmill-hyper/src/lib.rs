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
use std::pin::{pin, Pin};
use std::task::{Context, Poll};
use std::thread;
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

    #[cfg(feature = "server")]
    pub async fn accept(&self) -> Option<io::Result<TreadmillStream>> {
        let mut incoming = pin!(self.io.incoming());
        let stream = incoming.next().await?;
        Some(stream.map(|stream| TreadmillStream { stream }))
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
            trace!("Pending data to read");
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
        trace!("Writing");
        Pin::new(&mut self.stream).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        trace!("Flushing");
        Pin::new(&mut self.stream).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        trace!("Shutting down");
        Pin::new(&mut self.stream).poll_close(cx)
    }
}
