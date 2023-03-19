//! Works on things...?
use async_task::Runnable;
use crossbeam_deque::{Stealer, Worker};

pub fn make_workers(workers: usize) -> Vec<WorkerThread> {
    let mut result = (0..workers)
        .map(|_| WorkerThread::new())
        .collect::<Vec<_>>();

    for i in 0..workers {
        for j in 0..workers {
            if j == i {
                continue;
            }
            let stealer = result[j].queue.stealer();
            result[i].stealers.push(stealer);
        }
    }
    result
}

pub struct WorkerThread {
    queue: Worker<Runnable>,
    stealers: Vec<Stealer<Runnable>>,
}

// Should probably make a method to spawn the thread and handle tasky stuff, as well as a push
// method to push a task onto queue
impl WorkerThread {
    pub fn new() -> Self {
        let mut this = Self {
            queue: Worker::new_fifo(),
            stealers: vec![],
        };
        this.run();
        this
    }

    pub fn submit_task(&self, runnable: Runnable) {
        self.queue.push(runnable);
    }

    pub fn run(&self) {
        // here I want to start the queue processing. So I'll spawn a thread and then read from
        // queue or steal and then run `runnable.run()` on the tasks I get
        todo!();
    }
}
