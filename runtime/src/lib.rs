use std::future::Future;

mod handle;
mod next_frame;
mod runtime;
mod task;
mod task_waker;

pub use handle::JoinHandle;
pub use runtime::Runtime;

thread_local! {
    pub static RUNTIME: Runtime = Runtime::new();
}

pub fn runtime() -> Runtime {
    RUNTIME.with(|r| r.clone())
}

pub fn runtime_update() {
    runtime().runtime_update();
}

pub fn next_frame() -> impl Future<Output = ()> {
    runtime().next_frame()
}

pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + 'static,
    F::Output: 'static,
{
    runtime().spawn(future)
}
