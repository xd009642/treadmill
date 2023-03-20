use crate::worker::{make_workers, WorkerThread};
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
    static RUNTIME_CONTEXT: RefCell<Option<Runtime>> = RefCell::new(None)
}

/// Need to add some run queues, one for each worker, probably some sort of task construct
#[derive(Clone)]
pub struct Runtime {
    workers: Arc<Vec<WorkerThread>>,
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
        let workers = Arc::new(make_workers(self.workers));
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
    /// [link](https://docs.rs/futures-lite/latest/futures_lite/future/fn.block_on.html) was used
    /// as a starting point but has been adapted for the multi-threaded runtime I'm trying to make
    pub fn block_on<T: Send + 'static>(&self, future: impl Future<Output = T>) -> T {
        // Here we should probably create our worker threads, turn this task into a Runnable and
        // submit it
        //
        // The current future will execute on the current thread (and not move) and then all
        // subsequent spawned tasks will be ran in the spawned workers

        future::block_on(future)
    }

    pub fn spawn<F, T>(&self, future: F) -> Task<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        // Here I probably want to have some sort of RuntimeWorkers object containing the
        // references to workers which I can move into a schedule function and submit to
        // async_task::spawn. Then the object will pick a queue and pop the runnable into that
        // queue and that queue will handle running.
        //
        // Probably need a way to get a handle to the current runtime without needing to store a
        // reference to it around all the time - similar ergonomics to tokio with
        // `tokio::task::spawn`
        static QUEUE: Lazy<Sender<Runnable>> = Lazy::new(|| {
            let (tx, rx) = unbounded::<Runnable>();

            thread::spawn(move || {
                while let Ok(runnable) = rx.recv() {
                    trace!("Runnable is being scheduled");
                    let _ = catch_unwind(|| runnable.run());
                }
            });

            tx
        });

        let schedule = |runnable| QUEUE.send(runnable).unwrap();
        let (runnable, task) = async_task::spawn(future, schedule);

        runnable.schedule();
        task
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
    static QUEUE: Lazy<Sender<Runnable>> = Lazy::new(|| {
        let (tx, rx) = unbounded::<Runnable>();

        thread::spawn(move || {
            while let Ok(runnable) = rx.recv() {
                let _ = catch_unwind(|| runnable.run());
            }
        });

        tx
    });

    let schedule = |runnable| QUEUE.send(runnable).unwrap();
    let (runnable, task) = async_task::spawn(future, schedule);

    runnable.schedule();
    task
}
