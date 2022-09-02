use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};

use crate::handle::JoinHandle;

pub struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
    pub(super) callback_waker: Option<Waker>,
}
impl Task {
    pub(super) fn poll(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        Future::poll(self.future.as_mut(), cx)
    }

    pub(super) fn register_callback(&mut self, waker: Waker) {
        self.callback_waker = Some(waker);
    }
}

pub(crate) fn joinable<F>(future: F) -> (Rc<RefCell<Task>>, JoinHandle<F::Output>)
where
    F: Future + 'static,
    F::Output: 'static,
{
    let value = Rc::new(RefCell::new(None));

    let task = {
        let value = Rc::clone(&value);
        Rc::new(RefCell::new(Task {
            future: Box::pin(async move {
                let output = future.await;
                let mut value = value.borrow_mut();
                *value = Some(output);
            }),
            callback_waker: None,
        }))
    };

    let handle = JoinHandle {
        value,
        task: Rc::clone(&task),
    };

    (task, handle)
}
