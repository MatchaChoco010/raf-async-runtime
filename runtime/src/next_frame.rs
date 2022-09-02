use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::runtime::Runtime;

pub struct NextFrameFuture {
    called: bool,
    runtime: Runtime,
}
impl NextFrameFuture {
    pub(crate) fn new(runtime: Runtime) -> Self {
        Self {
            called: false,
            runtime,
        }
    }
}
impl Future for NextFrameFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.called {
            Poll::Ready(())
        } else {
            self.called = true;
            let waker = cx.waker().clone();
            self.runtime
                .shared
                .borrow_mut()
                .frame_change_wakers
                .push(waker);
            Poll::Pending
        }
    }
}
