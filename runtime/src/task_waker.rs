use std::mem::ManuallyDrop;
use std::rc::Rc;
use std::task::{RawWaker, RawWakerVTable, Waker};

use crate::runtime::Runtime;

pub(crate) struct TaskWaker {
    runtime: Runtime,
    task_id: usize,
}
impl TaskWaker {
    pub(crate) fn waker(runtime: Runtime, task_id: usize) -> Waker {
        let waker = Rc::new(TaskWaker { runtime, task_id });
        unsafe { Waker::from_raw(Self::into_raw_waker(waker)) }
    }

    pub(crate) fn wake_by_ref(this: &Rc<Self>) {
        // // waiting queueから取り出してrunning queueに入れる
        let task = this
            .runtime
            .shared
            .borrow_mut()
            .waiting_queue
            .remove(&this.task_id);
        if let Some(task) = task {
            this.runtime
                .shared
                .borrow_mut()
                .running_queue
                .push_back(task);
        }
    }

    // RawWakerを作る。
    // WakerはSync + Sendを要求するが、wasmではシングルスレッドなので、
    // Rcのポインタだけで良いということにする。
    unsafe fn into_raw_waker(this: Rc<Self>) -> RawWaker {
        unsafe fn raw_clone(ptr: *const ()) -> RawWaker {
            let ptr = ManuallyDrop::new(Rc::from_raw(ptr as *const TaskWaker));
            TaskWaker::into_raw_waker((*ptr).clone())
        }

        unsafe fn raw_wake(ptr: *const ()) {
            let ptr = Rc::from_raw(ptr as *const TaskWaker);
            TaskWaker::wake_by_ref(&ptr);
        }

        unsafe fn raw_wake_by_ref(ptr: *const ()) {
            let ptr = ManuallyDrop::new(Rc::from_raw(ptr as *const TaskWaker));
            TaskWaker::wake_by_ref(&ptr);
        }

        unsafe fn raw_drop(ptr: *const ()) {
            drop(Rc::from_raw(ptr as *const TaskWaker));
        }

        const VTABLE: RawWakerVTable =
            RawWakerVTable::new(raw_clone, raw_wake, raw_wake_by_ref, raw_drop);

        RawWaker::new(Rc::into_raw(this) as *const (), &VTABLE)
    }
}
