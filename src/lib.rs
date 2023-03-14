use std::{future::Future, panic::catch_unwind, thread};
use async_task::{Runnable, Task};
use once_cell::sync::Lazy;
use crossbeam_channel::{unbounded, Sender};


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
    return task;
}
