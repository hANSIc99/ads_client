use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use std::future::Future;
use std::thread;
use std::pin::Pin;
use crate::Handle;


pub struct CommandCleaner{
    waker           : Option<Arc<Mutex<Waker>>>,
    handle_register : Arc<Mutex<Vec<Handle>>>,
    interval        : u64,
    timeout         : Duration
}

impl CommandCleaner {
    pub fn new(interval : u64, timeout : u64, handle_register : Arc<Mutex<Vec<Handle>>>) -> CommandCleaner {
        CommandCleaner {
            waker               : None,
            handle_register     : handle_register,
            interval            : interval,
            timeout             : Duration::from_secs(timeout)
        }
    }
}

impl Future for CommandCleaner {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()>
    {
        if let Some(waker) = &self.waker {
            let mut waker = waker.lock().unwrap();

            if !waker.will_wake(cx.waker()) {
                *waker = cx.waker().clone();
            }
        } else {
            let waker = Arc::new(Mutex::new(cx.waker().clone()));
            self.waker = Some(waker.clone());
        }

        let waker = Arc::clone(&self.waker.as_ref().unwrap());
        let interval = self.interval;

        thread::spawn(move || {

            thread::sleep(Duration::from_secs(interval)); // Checke every second for stale handles
            let waker = waker.lock().unwrap();
            waker.wake_by_ref();
        });

        let a_handles = Arc::clone(&self.handle_register);

        let mut handles = a_handles.lock().expect("Threading Error");
        let now = Instant::now();

        handles.retain( |hdl| {
            // now() - hdl.timestamp < timeout == VALID
            let test = now - hdl.timestamp < self.timeout;
            // if !test {
            //     println!("Handle invalidated");
            // }
            test
        });

        Poll::Pending
    }
}