use std::future::Future;
use std::task::{Context, Poll, Waker, RawWaker, RawWakerVTable};
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::VecDeque;
use crate::utils::*;

use anyhow;
use web_sys::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

static mut GLOBAL_TASK_QUEUE : Option<VecDeque<Box<dyn FnOnce() -> ()>>> = None;

type WakerData = *const ();

struct WakerDataImpl {
    awoken_flag: bool,
    polling_flag: bool
}

unsafe fn clone(ptr : WakerData) -> RawWaker {
    let flags : Rc<RefCell<WakerDataImpl>> = Rc::from_raw(ptr as *const RefCell<WakerDataImpl>);
    let flags_clone = flags.clone();
    Rc::into_raw(flags);
    return my_raw_waker(flags_clone);
}

unsafe fn wake(ptr : WakerData) {
    wake_by_ref(ptr);
    drop(ptr);
}

unsafe fn wake_by_ref(ptr : WakerData) {
    let flags : Rc<RefCell<WakerDataImpl>> = Rc::from_raw(ptr as *const RefCell<WakerDataImpl>);
    flags.borrow_mut().awoken_flag = true;    

    if !flags.borrow().polling_flag {
        respawn();
    }

    Rc::into_raw(flags);
}

unsafe fn drop(ptr : WakerData) {
    let _ : Rc<RefCell<WakerDataImpl>> = Rc::from_raw(ptr as *const RefCell<WakerDataImpl>);
}

static GLOBAL_WAKER_VTABLE : RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

fn my_raw_waker(flags : Rc<RefCell<WakerDataImpl>>) -> RawWaker {
    RawWaker::new(Rc::into_raw(flags) as *const (), &GLOBAL_WAKER_VTABLE)
}

pub async fn execute_queue() {
    unsafe {
        if let Some(queue) = &mut GLOBAL_TASK_QUEUE {
            let len = queue.len();

            for i in 0..len {
                if let Some(item) = queue.pop_back() {
                    item();
                }
            }
        }
    }
}

pub fn respawn() {
    let window = window()
        .ok_or(anyhow::anyhow!("Failed to get window!"))
        .unwrap();

    let closure = Closure::once_into_js(move || {
        wasm_bindgen_futures::spawn_local(execute_queue());
    });

    window
        .set_timeout_with_callback(closure.as_ref().unchecked_ref())
        .unwrap();
}

pub fn execute_pin<F>(mut future: std::pin::Pin<std::boxed::Box<F>>)
where
    F: Future<Output = ()> + 'static,
{
    let flags = Rc::new(RefCell::new(WakerDataImpl { awoken_flag : false, polling_flag : true }));

    let waker = unsafe {
        let raw_waker = my_raw_waker(flags.clone());
        Waker::from_raw(raw_waker)
    };

    let mut context = &mut Context::from_waker(&waker);

    let mut counter = 0;

    loop {
        let future_mut = future.as_mut();
        match future_mut.poll(&mut context) {
            Poll::Ready(v) => return v,
            Poll::Pending => {
                if flags.borrow().awoken_flag && counter < 128 {
                    flags.borrow_mut().awoken_flag = false;
                    counter += 1;   
                }
                else {
                    break;
                }
            },
        }
    }

    flags.borrow_mut().polling_flag = false;

    let poll = move || {
        execute_pin(future);
    };

    unsafe {
        if let None = &mut GLOBAL_TASK_QUEUE {
            GLOBAL_TASK_QUEUE = Some( VecDeque::<Box<dyn FnOnce() -> ()>>::new() );
        }

        if let Some(queue) = &mut GLOBAL_TASK_QUEUE {
            queue.push_back(Box::new(poll));
        }
    }

    if flags.borrow().awoken_flag {
        respawn();
    }
}

pub fn execute<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    let future = Box::pin(future);
    execute_pin(future);
}

pub async fn yield_now() {
    struct YieldNow {
        yielded: bool,
    }

    impl Future for YieldNow {
        type Output = ();

        fn poll(mut self: std::pin::Pin<&mut Self>, context: &mut Context<'_>) -> Poll<()> {
            if self.yielded {
                return Poll::Ready(());
            }

            self.yielded = true;
            context.waker().wake_by_ref();
            Poll::Pending
        }
    }

    YieldNow { yielded: false }.await
}
