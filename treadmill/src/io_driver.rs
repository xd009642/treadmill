//! The `io_driver::Driver` will use mio to handle different forms of IO. mio abstracts around
//! different non-blocking APIs across platforms like Windows and Unix and Mac and means we can
//! avoid writing a bunch of platform specific epoll or kqueue code and instead just focus on
//! runtime like things!
//!
//! https://docs.rs/mio/latest/mio/guide/index.html

use mio::{Events, Interest, Poll, Token};
use std::io;
use std::time::Duration;
use tracing::trace;

pub struct Driver {
    poll: Poll,
    events: Events,
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

impl Driver {
    pub fn new() -> io::Result<Self> {
        Self::new_with_capacity(512)
    }

    pub fn new_with_capacity(events: usize) -> io::Result<Self> {
        let poll = Poll::new()?;
        let events = Events::with_capacity(events);

        Ok(Self { poll, events })
    }

    pub fn poll_io(&mut self, max_wait: Option<Duration>) {
        let poll_res = self.poll.poll(&mut self.events, max_wait);

        match poll_res {
            Ok(_) => {}
            Err(e) => trace!("Error polling io driver: {}", e),
        }

        // Some events might have still succeeded
        for event in &self.events {
            // Dispatch them?
            let token = event.token();
            trace!("Got event for Token {:?}", token);
        }
    }
}
