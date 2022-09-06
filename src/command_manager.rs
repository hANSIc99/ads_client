use std::time::Instant;
use std::future::Future;
use std::task::{Context, Poll};
use std::sync::{Arc, Mutex};
use std::pin::Pin;
use bytes::Bytes;
use crate::{AdsError, Result, Handle};

pub const ADSERR_CLIENT_SYNCTIMEOUT : u32 = 0x745;

pub struct CommandManager{
    now             : Instant,
    timeout         : u64,
    invoke_id       : u32,
    handle_register : Arc<Mutex<Vec<Handle>>>
}

impl CommandManager {
    pub fn new(timeout : u64, invoke_id : u32, handle_register : Arc<Mutex<Vec<Handle>>>) -> CommandManager {
        CommandManager {
            now             : Instant::now(),
            timeout         : timeout,
            invoke_id       : invoke_id,
            handle_register : handle_register
        }
    }
}

impl Future for CommandManager {
    type Output = Result<Bytes>;
   
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<Bytes>>{
        if self.now.elapsed().as_secs() > self.timeout{
            // Does this still work if the Future os moved between threads/cores?
            // https://doc.rust-lang.org/std/time/struct.Instant.html
            Poll::Ready(Err(Box::new(AdsError{n_error : ADSERR_CLIENT_SYNCTIMEOUT})))
        } else {
            let a_handles = Arc::clone(&self.handle_register);

            let mut handles = a_handles.lock().expect("Threading Error");
            let mut _iter = handles.iter_mut();
            let pos = _iter.position( | hdl | {
                // Proceed when the invoke ID is match and data is attached
                hdl.invoke_id == self.invoke_id && hdl.data.is_some()
            });

            match pos {
                Some(_pos) => {
                    //
                    let hdl = handles.swap_remove(pos.unwrap());
                    return Poll::Ready(Ok(hdl.data.unwrap()))
                },
                None => {
                    cx.waker().wake_by_ref();
                    return Poll::Pending
                }
            }
        }
    }
}