use std::future::Future;
use std::pin::Pin;
use std::sync::Mutex;
use std::task::{Context, Poll};
use std::thread;

struct Handle<R>
where
    R: Send + 'static,
{
    join_handle: Option<thread::JoinHandle<R>>,
    f: Option<Box<dyn FnOnce() -> R + Send + 'static>>,
}

pub struct JoinHandle<R>
where
    R: Send + 'static,
{
    handle: Mutex<Handle<R>>,
}

impl<R> JoinHandle<R>
where
    R: Send + 'static,
{
    pub(crate) fn new<F>(f: F) -> Self
    where
        F: FnOnce() -> R + Send + 'static,
    {
        let handle = Handle {
            join_handle: None,
            f: Some(Box::new(f)),
        };
        Self {
            handle: Mutex::new(handle),
        }
    }
}

impl<T> Future for JoinHandle<T>
where
    T: Send + 'static,
{
    type Output = thread::Result<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut h = self.handle.lock().unwrap(); // todo
        if h.join_handle.is_none() && h.f.is_some() {
            let f = h.f.take().unwrap();
            let waker = cx.waker().clone();
            h.join_handle = Some(thread::spawn(move || {
                let res = f();
                waker.wake();
                res
            }));
            Poll::Pending
        } else if let Some(handle) = h.join_handle.take() {
            // We should be finished
            Poll::Ready(handle.join())
        } else {
            Poll::Ready(Err(Box::new(())))
        }
    }
}
