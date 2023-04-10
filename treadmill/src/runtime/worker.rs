//! Works on things...?
use super::io::Driver;
use async_task::Runnable;
use async_task::Task;
use core::future::Future;
use crossbeam_deque::{Steal, Stealer, Worker};
use std::sync::Arc;
use std::{io, thread};
use tracing::trace;

#[derive(Clone)]
pub struct WorkerPool {
    workers: Arc<Vec<Arc<WorkerThread>>>,
    driver: Arc<Driver>,
}

impl WorkerPool {
    pub fn new(len: usize, enable_work_stealing: bool) -> Self {
        assert_ne!(len, 0);

        let driver = Driver::new().expect("Unable to create IO Driver");

        Self {
            workers: Arc::new(make_workers(len, enable_work_stealing)),
            driver: driver.into(),
        }
    }

    pub fn empty() -> Self {
        let driver = Driver::new_with_capacity(0).expect("Unable to create IO Driver");
        Self {
            workers: Default::default(),
            driver: driver.into(),
        }
    }

    pub fn spawn<F, T>(&self, future: F) -> Task<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let worker_index = fastrand::usize(0..self.workers.len());
        trace!("Spawning task to worker[{}]", worker_index);
        self.workers[worker_index].submit_task(future)
    }

    pub fn spawn_on_worker<F, T>(&self, future: F, worker_index: usize) -> Task<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        trace!("Spawning task to worker[{}]", worker_index);
        self.workers[worker_index].submit_task(future)
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.workers.is_empty()
    }

    pub fn io_driver(&self) -> Arc<Driver> {
        self.driver.clone()
    }
}

fn make_workers(workers: usize, enable_work_stealing: bool) -> Vec<Arc<WorkerThread>> {
    let (txs, mut rxs): (Vec<_>, Vec<_>) = (0..workers).map(|_| WorkerThread::new()).unzip();

    for i in 0..workers {
        if enable_work_stealing {
            for j in 0..workers {
                if j == i {
                    continue;
                }
                let stealer = txs[j].queue.stealer();
                rxs[i].stealers.push(stealer);
                rxs[i].id = i;
            }
        }
        trace!("Starting worker {} task receiving queue", i);
        rxs[i].clone().run();
    }
    txs
}

pub struct WorkerThread {
    queue: Worker<Runnable>,
}

#[derive(Clone)]
pub struct TaskReceiver {
    id: usize,
    queue_out: Stealer<Runnable>,
    stealers: Vec<Stealer<Runnable>>,
}

// Should probably make a method to spawn the thread and handle tasky stuff, as well as a push
// method to push a task onto queue
impl WorkerThread {
    pub fn new() -> (Arc<Self>, TaskReceiver) {
        let queue = Worker::new_fifo();
        let queue_out = queue.stealer();
        let tx = Arc::new(Self { queue });
        let rx = TaskReceiver {
            id: 0,
            queue_out,
            stealers: vec![],
        };
        (tx, rx)
    }

    pub fn submit_task<F, T>(&self, future: F) -> Task<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let schedule = |runnable| self.queue.push(runnable);
        let (runnable, task) = unsafe { async_task::spawn_unchecked(future, schedule) };
        runnable.schedule();
        task
    }
}

impl TaskReceiver {
    pub fn run(self) {
        // detach go brrrrrrrrr
        thread::spawn(move || {
            // here I want to start the queue processing. So I'll spawn a thread and then read from
            // queue or steal and then run `runnable.run()` on the tasks I get
            loop {
                if let Steal::Success(runnable) = self.queue_out.steal() {
                    trace!("Running {}", self.id);
                    runnable.run();
                } else {
                    for stealer in &self.stealers {
                        if let Steal::Success(runnable) = stealer.steal() {
                            trace!("Stole a task");
                            runnable.run();
                            break;
                        }
                    }
                }
            }
        });
    }
}
