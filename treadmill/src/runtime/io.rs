//! The `io_driver::Driver` will use mio to handle different forms of IO. mio abstracts around
//! different non-blocking APIs across platforms like Windows and Unix and Mac and means we can
//! avoid writing a bunch of platform specific epoll or kqueue code and instead just focus on
//! runtime like things!
//!
//! https://docs.rs/mio/latest/mio/guide/index.html

use mio::{event::Source, Events, Interest, Poll, Token};
use std::io;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use tracing::{error, trace};

pub struct Driver {
    poll: Poll,
    events: Events,
    current_token: AtomicUsize,
}

pub struct IoHandle<S: Source> {
    source: S,
    registry: mio::Registry,
}

impl<S> IoHandle<S>
where
    S: Source,
{
    pub fn source(&self) -> &S {
        &self.source
    }
}

impl<E: Source> Drop for IoHandle<E> {
    fn drop(&mut self) {
        if let Err(e) = self.source.deregister(&self.registry) {
            error!("Failed to deregister IoHandle: {}", e);
        }
    }
}

// Okay the vague plan...
//
// Create new IO Objects and submit them to this maybe using the Source trait as a trait bound.
//
// Mio has a waker so we can pop into the poll and maybe have IO driver thread getting the events,
// getting data from the events and notifying the source objects it's there... But then how do we
// send the data over? A channel? Holding a reference to an object made from driver that just
// receives the data from that one source?
//
// How do we handle re-using tokens after something disappears?
//
// Okay general gist of how I understand the IO stuff will work.
//
// When we create an IO object the object will:
//
// 1. Grab the handle to current runtime and get the IO driver
// 2. Create it's IO object i.e. mio::TcpListener
// 3. Register this object with mio with the provided interest (Read/Write)
// 4. The object will get a token which we use to tell the object things are ready
//
// Then when we poll mio and get the events which are ready we:
//
// 1. Wake up those futures - I guess the token maps to the waker or thread park/unpark
// 2. The future will attempt a read or write and should hopefully have data
// 3. If it returns interrupted or would block things are good just a false wakeup - other errors
//    are real
// 4. If object is done, unregister the token freeing up the ID for future tokens

impl Driver {
    pub fn new() -> io::Result<Self> {
        Self::new_with_capacity(512)
    }

    /// Returns an IO driver and a Registry. The Registry just contains a independently owned
    /// version of the struct IO is registered with and allows for types to register themselves
    /// without needing to access the full driver - and also then be moved to other threads easily.
    pub fn new_with_capacity(events: usize) -> io::Result<Self> {
        let poll = Poll::new()?;
        let events = Events::with_capacity(events);

        let current_token = AtomicUsize::new(0);
        let driver = Self {
            poll,
            events,
            current_token,
        };

        Ok(driver)
    }

    pub fn poll_io(&mut self, max_wait: Option<Duration>) {
        let poll_res = self.poll.poll(&mut self.events, max_wait);

        match poll_res {
            Ok(_) => {}
            Err(e) => error!("Error polling io driver: {}", e),
        }

        // Some events might have still succeeded
        for event in &self.events {
            // Dispatch them?
            let token = event.token();
            trace!("Got event for Token {:?}", token);

            // Now we need to wake up our future somehow?
        }
    }

    fn get_token(&self) -> Token {
        let id = self.current_token.fetch_add(1, Ordering::SeqCst);
        Token(id)
    }

    pub fn create_io_source<S>(
        &self,
        mut source: S,
        interest: mio::Interest,
    ) -> io::Result<IoHandle<S>>
    where
        S: Source,
    {
        let token = self.get_token();
        let registry = self.poll.registry().try_clone()?;
        registry.register(&mut source, token, interest)?;
        Ok(IoHandle { source, registry })
    }
}
