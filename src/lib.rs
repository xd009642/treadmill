use crate::worker::WorkerThread;
use async_task::{Runnable, Task};
use core::pin::{pin, Pin};
use core::task::{Context, Poll};
use crossbeam_channel::{unbounded, Sender};
use once_cell::sync::Lazy;
use parking::Parker;
use std::cell::RefCell;
use std::task::Waker;
use std::{future::Future, panic::catch_unwind, thread};
use waker_fn::waker_fn;

pub mod worker;

/// Need to add some run queues, one for each worker, probably some sort of task construct
pub struct Runtime {
    workers: Vec<WorkerThread>,
}

// Creates a parker and an associated waker that unparks it.
fn parker_and_waker() -> (Parker, Waker) {
    let parker = Parker::new();
    let unparker = parker.unparker();
    let waker = waker_fn(move || {
        unparker.unpark();
    });
    (parker, waker)
}

impl Runtime {
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

        // Pin the future on the stack.
        let mut future = pin!(future);

        thread_local! {
            // Cached parker and waker for efficiency.
            static CACHE: RefCell<(Parker, Waker)> = RefCell::new(parker_and_waker());
        }

        CACHE.with(|cache| {
            // Try grabbing the cached parker and waker.
            match cache.try_borrow_mut() {
                Ok(cache) => {
                    // Use the cached parker and waker.
                    let (parker, waker) = &*cache;
                    let cx = &mut Context::from_waker(&waker);

                    // Keep polling until the future is ready.
                    loop {
                        match future.as_mut().poll(cx) {
                            Poll::Ready(output) => return output,
                            Poll::Pending => parker.park(),
                        }
                    }
                }
                Err(_) => {
                    // TODO tokio doesn't like recursive block_on's because of deadlock risks? Can
                    // I recreate this issue and then make a decision on whether this should be
                    // allowed.

                    // Looks like this is a recursive `block_on()` call.
                    // Create a fresh parker and waker.
                    let (parker, waker) = parker_and_waker();
                    let cx = &mut Context::from_waker(&waker);

                    // Keep polling until the future is ready.
                    loop {
                        match future.as_mut().poll(cx) {
                            Poll::Ready(output) => return output,
                            Poll::Pending => parker.park(),
                        }
                    }
                }
            }
        })
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
