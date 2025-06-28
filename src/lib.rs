mod oneshot;
use std::ops::{Deref, DerefMut};

#[repr(C)]
#[derive(Debug)]
pub struct OwnedMutRefWaiter<'a, T: Send>(oneshot::Receiver<()>, &'a mut T);

impl<T: Send> OwnedMutRefWaiter<'_, T> {
    pub fn wait(self) {
        let (done, _ui): (oneshot::Receiver<()>, &mut T) = unsafe { std::mem::transmute(self) };
        done.recv();
    }
}

impl<T: Send> Drop for OwnedMutRefWaiter<'_, T> {
    fn drop(&mut self) {
        eprintln!("OwnedMutRefWaiter needs to be waited on");
        std::process::abort();
    }
}

#[derive(Debug)]
pub struct OwnedMutRef<T: Send> {
    _done: oneshot::Sender<()>,
    value: *mut T,
}

unsafe impl<T: Send> Send for OwnedMutRef<T> {}
unsafe impl<T: Send + Sync> Sync for OwnedMutRef<T> {}

impl<T: Send> Deref for OwnedMutRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.value }
    }
}

impl<T: Send> DerefMut for OwnedMutRef<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.value }
    }
}

impl<T: Send> Drop for OwnedMutRef<T> {
    fn drop(&mut self) {}
}

impl<T: Send> OwnedMutRef<T> {
    ///```rust
    /// use owned_mut_ref::OwnedMutRef;
    /// use std::sync::mpsc;
    ///
    /// fn main() {
    ///     // create a mutable reference
    ///     let mut x = 1;
    ///     let mut_ref = &mut x;
    ///
    ///     // create the owned_mut_ref and the owned_mut_ref_waiter
    ///     let (owned_mut, waiter) = OwnedMutRef::new(mut_ref);
    ///
    ///     // now do wathever you want with your owned_mut_ref
    ///     // like send it through a channel
    ///     let (tx, rx) = mpsc::channel();
    ///     tx.send(owned_mut).unwrap();
    ///     let mut received_owned_mut = rx.recv().unwrap();
    ///     // then use it on the other end
    ///     *received_owned_mut += 1;
    ///
    ///     // dropping the received_owned_mut will allow the waiter to continue
    ///     drop(received_owned_mut);
    ///
    ///     // you must wait on the waiter
    ///     // dropping the waiter will abort the program
    ///     waiter.wait();
    ///
    ///     // once you called wait the value is free to be used again
    ///     println!("{x}");
    /// }
    ///```
    pub fn new(value: &mut T) -> (Self, OwnedMutRefWaiter<T>) {
        let (done_tx, done_rx) = oneshot::channel();
        (
            Self {
                _done: done_tx,
                value: value as *mut T,
            },
            OwnedMutRefWaiter(done_rx, value),
        )
    }
}
