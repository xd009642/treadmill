use mio::net;
use mio::{Events, Interest, Poll, Token};
use std::io;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;

pub struct TcpListener {
    listener: net::TcpListener,
}

fn for_each_addr<A, F, T>(addrs: A, f: F) -> io::Result<T>
where
    A: ToSocketAddrs,
    F: Fn(SocketAddr) -> io::Result<T>,
{
    let addr = match addrs.to_socket_addrs() {
        Ok(addrs) => addrs,
        Err(e) => return Err(e),
    };
    if let Some(val) = addr.map(|x| f(x)).find(|x| x.is_ok()) {
        val
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "could not resolve to any addresses",
        ))
    }
}

impl TcpListener {
    pub async fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<TcpListener> {
        let listener = for_each_addr(addr, net::TcpListener::bind)?;
        // We probably want an IO driver struct in the runtime to handle the poll/events and
        // registry
        todo!();
    }

    pub async fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
        let (stream, addr) = net::TcpListener::accept(&self.listener)?;
        todo!()
    }
}

pub struct TcpStream {}

impl TcpStream {
    pub async fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<TcpStream> {
        todo!()
    }
}
