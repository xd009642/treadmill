//! The `io_driver::Driver` will use mio to handle different forms of IO. mio abstracts around
//! different non-blocking APIs across platforms like Windows and Unix and Mac and means we can
//! avoid writing a bunch of platform specific epoll or kqueue code and instead just focus on
//! runtime like things!
//!
//! https://docs.rs/mio/latest/mio/guide/index.html

use mio::{Events, Interest, Poll, Token};
use std::io;

pub struct Driver {
    poll: Poll,
    events: Events,
}

impl Driver {
    pub fn new() -> io::Result<Self> {
        Self::new_with_capacity(512)
    }

    pub fn new_with_capacity(events: usize) -> io::Result<Self> {
        let poll = Poll::new()?;
        let events = Events::with_capacity(events);

        Ok(Self { poll, events })
    }
}
