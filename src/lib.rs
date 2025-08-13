use std::{
    ops::{Deref, DerefMut},
    pin::Pin,
    sync::{Arc, Condvar, Mutex},
    task::{Context, Poll, Waker},
};

#[derive(Debug)]
#[allow(dead_code)] // suppress the warning about the unused &'a mut T
pub struct OwnedMutRefWaiter<'a, T: Send>(Arc<(Condvar, Mutex<(Option<Waker>, bool)>)>, &'a mut T);

impl<T: Send> OwnedMutRefWaiter<'_, T> {
    pub fn wait(self) {
        #[allow(let_underscore_lock)]
        let _ = self.0.0.wait_while(self.0.1.lock().unwrap(), |x| !x.1);
    }
    /// returns true if the waiter is ready to be dropped, false otherwise
    pub fn try_wait(&self) -> bool {
        self.0.1.lock().unwrap().1
    }
}

impl<T: Send> Future for OwnedMutRefWaiter<'_, T> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut lock = self.0.1.lock().unwrap();
        if lock.1 {
            Poll::Ready(())
        } else {
            lock.0 = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl<T: Send> Drop for OwnedMutRefWaiter<'_, T> {
    fn drop(&mut self) {
        if !self.0.1.lock().unwrap().1 {
            eprintln!("OwnedMutRefWaiter needs to be waited on");
            std::process::abort();
        }
    }
}

#[derive(Debug)]
pub struct OwnedMutRef<T: Send> {
    done: Arc<(Condvar, Mutex<(Option<Waker>, bool)>)>,
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
    fn drop(&mut self) {
        let mut lock = self.done.1.lock().unwrap();
        lock.1 = true;
        lock.0.take().map(|waker| waker.wake());
        self.done.0.notify_one();
    }
}

impl<T: Send> OwnedMutRef<T> {
    ///```rust
    /// use owned_mut_ref::OwnedMutRef;
    /// use std::sync::mpsc;
    ///
    /// fn main() {
    ///     // WARNING: KILLING THE THREAD THAT OWNS THE MUTABLE REFERENCE IS UB
    ///
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
    ///     // dropping the waiter before the owned_mut is dropped will std::process:abort
    ///     waiter.wait();
    ///     // for async code
    ///     //waiter.await
    ///
    ///     // once you called wait the value is free to be used again
    ///     println!("{x}");
    /// }
    ///```

    pub fn new(value: &mut T) -> (Self, OwnedMutRefWaiter<T>) {
        let done = Arc::new((Condvar::new(), Mutex::new((None, false))));
        (
            Self {
                done: done.clone(),
                value: value as *mut T,
            },
            OwnedMutRefWaiter(done, value),
        )
    }
}
