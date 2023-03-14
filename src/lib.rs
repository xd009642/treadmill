use async_task::{Runnable, Task};
use crossbeam_channel::{unbounded, Sender};
use once_cell::sync::Lazy;
use std::{future::Future, panic::catch_unwind, thread};

/// Need to add some run queues, one for each worker, probably some sort of task construct
pub struct Runtime {}

impl Runtime {
    pub fn block_on<T: Send + 'static>(&self, _task: impl Future<Output = T>) -> T {
        todo!();
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
