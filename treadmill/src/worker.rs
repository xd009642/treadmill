//! Works on things...?
use async_task::Runnable;
use async_task::Task;
use core::future::Future;
use crossbeam_deque::{Steal, Stealer, Worker};
use std::sync::Arc;
use std::thread;
use tracing::trace;

#[derive(Clone)]
pub struct WorkerPool {
    workers: Arc<Vec<Arc<WorkerThread>>>,
}

impl WorkerPool {
    pub fn new(len: usize) -> Self {
        assert_ne!(len, 0);

        Self {
            workers: Arc::new(make_workers(len)),
        }
    }

    pub fn empty() -> Self {
        Self {
            workers: Default::default(),
        }
    }

    pub fn spawn<F, T>(&self, future: F) -> Task<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let worker_index = fastrand::usize(0..self.workers.len());
        self.workers[worker_index].submit_task(future)
    }
}

pub fn make_workers(workers: usize) -> Vec<Arc<WorkerThread>> {
    let (mut txs, rxs): (Vec<_>, Vec<_>) = (0..workers).map(|_| WorkerThread::new()).unzip();

    for i in 0..workers {
        for j in 0..workers {
            if j == i {
                continue;
            }
            let stealer = txs[j].queue.stealer();
            rxs[i].stealers.push(stealer);
        }
        trace!("Starting worker {} task receiving queue", i);
        rxs[i].run();
    }
    txs
}

pub struct WorkerThread {
    queue: Worker<Runnable>,
}

pub struct TaskReceiver {
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
        let (runnable, task) = async_task::spawn(future, schedule);
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
                    runnable.run();
                }
            }
        });
    }
}
