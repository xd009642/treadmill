//! In theory it's very easy to get your own asynchronous executor to drive hyper. So lets see if
//! that's the case...
use hyper::rt::Executor;
use std::future::Future;

#[derive(Clone, Copy)]
pub struct TreadmillExecutor;

impl<F> Executor<F> for TreadmillExecutor
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    fn execute(&self, fut: F) {
        treadmill::spawn(fut).detach();
    }
}

