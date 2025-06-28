mod oneshot;
use std::ops::{Deref, DerefMut};

#[repr(C)]
#[derive(Debug)]
pub struct SyncMutRefWaiter<'a, T: Send + Sync>(oneshot::Receiver<()>, &'a mut T);

impl<T: Send + Sync> SyncMutRefWaiter<'_, T> {
    pub fn wait(self) {
        let (done, _ui): (oneshot::Receiver<()>, &mut T) = unsafe { std::mem::transmute(self) };
        done.recv();
    }
}

impl<T: Send + Sync> Drop for SyncMutRefWaiter<'_, T> {
    fn drop(&mut self) {
        eprintln!("SyncMutRefWaiter needs to be waited on");
        std::process::abort();
    }
}

#[derive(Debug)]
pub struct SyncMutRef<T: Send + Sync> {
    _done: oneshot::Sender<()>,
    value: *mut T,
}

unsafe impl<T: Send + Sync> Send for SyncMutRef<T> {}
unsafe impl<T: Send + Sync> Sync for SyncMutRef<T> {}

impl<T: Send + Sync> Deref for SyncMutRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.value }
    }
}

impl<T: Send + Sync> DerefMut for SyncMutRef<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.value }
    }
}

impl<T: Send + Sync> Drop for SyncMutRef<T> {
    fn drop(&mut self) {}
}

impl<T: Send + Sync> SyncMutRef<T> {
    ///```rust
    /// use std::sync::mpsc;
    /// use sync_mut_ref::SyncMutRef;
    ///
    /// // create a mutable reference
    /// let mut x = 1;
    /// let mut_ref = &mut x;
    ///
    /// // create the sync_mut_ref and the sync_mut_ref waiter
    /// let (send_mut, waiter) = SyncMutRef::new(mut_ref);
    ///
    /// // now do wathever you want with your sync_mut_ref
    /// // like send it through a channel
    /// let (tx, rx) = mpsc::channel();
    /// tx.send(send_mut).unwrap();
    /// let mut received_sync_mut = rx.recv().unwrap();
    /// // then use it on the other end
    /// *received_sync_mut += 1;
    ///
    /// // dropping the received_sync_mut will allow the waiter to continue
    /// drop(received_sync_mut);
    ///
    /// // you must wait on the waiter
    /// // dropping the waiter will abort the program
    /// waiter.wait();
    ///
    /// // once you called wait the value is free to be used again
    /// println!("{x}");
    ///```
    pub fn new(value: &mut T) -> (Self, SyncMutRefWaiter<T>) {
        let (done_tx, done_rx) = oneshot::channel();
        (
            Self {
                _done: done_tx,
                value: value as *mut T,
            },
            SyncMutRefWaiter(done_rx, value),
        )
    }
}
