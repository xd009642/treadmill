use crate::worker::WorkerPool;
use async_task::{Runnable, Task};
use crossbeam_channel::{unbounded, Sender};
use futures_lite::future;
use once_cell::sync::Lazy;
use std::cell::RefCell;
use std::sync::Arc;
use std::thread_local;
use std::{future::Future, panic::catch_unwind, thread};
use tracing::trace;

pub mod worker;

thread_local! {
    static RUNTIME: RefCell<Runtime> = RefCell::new(Runtime {
        workers: WorkerPool::empty()
    })
}

/// Need to add some run queues, one for each worker, probably some sort of task construct
#[derive(Clone)]
pub struct Runtime {
    pub(crate) workers: WorkerPool,
}

pub struct RuntimeBuilder {
    workers: usize,
}

impl RuntimeBuilder {
    pub fn new() -> Self {
        Self {
            workers: num_cpus::get_physical(),
        }
    }

    pub fn num_workers(&mut self, workers: usize) -> &mut Self {
        self.workers = workers;
        self
    }

    pub fn build(self) -> Runtime {
        let workers = WorkerPool::new(self.workers);
        Runtime { workers }
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::builder().build()
    }
}

impl Runtime {
    pub fn builder() -> RuntimeBuilder {
        RuntimeBuilder::new()
    }

    /// This blocks the current thread waiting for the future to complete. The futures-lite
    /// implementation
    /// [link](https://docs.rs/futures-lite/latest/futures_lite/future/fn.block_on.html) is called
    /// to set up the waker function and parker and call the future poll methods manually.
    ///
    /// What this function does on top of this to necessitate it's existence is initialise the
    /// thread local handle to the runtime so that `treadmill::spawn` works and people don't need
    /// to keep around a handle to the runtime in order to use it!
    pub fn block_on<T: Send + 'static>(&self, future: impl Future<Output = T>) -> T {
        // Here we should probably create our worker threads, turn this task into a Runnable and
        // submit it
        //
        // The current future will execute on the current thread (and not move) and then all
        // subsequent spawned tasks will be ran in the spawned workers

        RUNTIME.with(|f| f.replace(self.clone()));

        future::block_on(future)
    }

    pub fn spawn<F, T>(&self, future: F) -> Task<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        self.workers.spawn(future)
    }

    pub fn current() -> Runtime {
        todo!()
    }
}

pub fn spawn<F, T>(future: F) -> Task<T>
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    match RUNTIME.try_with(|rt| rt.borrow().spawn(future)) {
        Ok(t) => t,
        Err(_e) => panic!("No runtime exists!"),
    }
}
