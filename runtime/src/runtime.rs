use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};
use std::future::Future;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::handle::JoinHandle;
use crate::task::*;
use crate::task_waker::TaskWaker;

pub(crate) struct Shared {
    pub(crate) frame_change_wakers: Vec<Waker>,
    pub(crate) running_queue: VecDeque<Rc<RefCell<Task>>>,
    pub(crate) waiting_queue: BTreeMap<usize, Rc<RefCell<Task>>>,
    task_counter: usize,
}

#[derive(Clone)]
pub struct Runtime {
    pub(crate) shared: Rc<RefCell<Shared>>,
}
impl Runtime {
    pub(crate) fn new() -> Self {
        let runtime = Self {
            shared: Rc::new(RefCell::new(Shared {
                frame_change_wakers: vec![],
                running_queue: VecDeque::new(),
                waiting_queue: BTreeMap::new(),
                task_counter: 0,
            })),
        };
        runtime.run();
        runtime
    }

    pub fn runtime_update(&self) {
        let wakers = self
            .shared
            .borrow_mut()
            .frame_change_wakers
            .drain(..)
            .collect::<Vec<_>>();
        for w in wakers {
            w.wake_by_ref();
        }

        'current_frame: loop {
            let task = self.shared.borrow_mut().running_queue.pop_front();

            match task {
                None => break 'current_frame,
                Some(task) => {
                    let id = {
                        let id = self.shared.borrow().task_counter;
                        self.shared.borrow_mut().task_counter += 1;
                        id
                    };

                    let waker = TaskWaker::waker(self.clone(), id);
                    let mut cx = Context::from_waker(&waker);

                    let result = task.borrow_mut().poll(&mut cx);
                    match result {
                        Poll::Ready(()) => {
                            // taskの完了をJoinHandleに通知する
                            task.borrow_mut()
                                .callback_waker
                                .iter()
                                .for_each(|w| w.wake_by_ref());
                        }
                        Poll::Pending => {
                            self.shared.borrow_mut().waiting_queue.insert(id, task);
                        }
                    }
                }
            }
        }
    }

    pub fn spawn<F>(&self, future: F) -> JoinHandle<F::Output>
    where
        F: Future + 'static,
        F::Output: 'static,
    {
        let (task, handle) = joinable(future);
        self.shared.borrow_mut().running_queue.push_back(task);
        handle
    }

    pub fn next_frame(&self) -> impl Future<Output = ()> {
        crate::next_frame::NextFrameFuture::new(self.clone())
    }

    fn run(&self) {
        let runtime = self.clone();

        fn window() -> web_sys::Window {
            web_sys::window().expect("no global `window` exists")
        }

        fn request_animation_frame(f: &Closure<dyn FnMut()>) {
            window()
                .request_animation_frame(f.as_ref().unchecked_ref())
                .expect("should register `requestAnimationFrame` OK");
        }

        let f = Rc::new(RefCell::new(None));
        let g = f.clone();

        *g.borrow_mut() = Some(Closure::new(move || {
            runtime.runtime_update();
            request_animation_frame(f.borrow().as_ref().unwrap());
        }));

        request_animation_frame(g.borrow().as_ref().unwrap());
    }
}
