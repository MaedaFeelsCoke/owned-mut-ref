use std::sync::{Arc, Condvar, Mutex};

#[derive(Debug)]
pub struct Sender<T>(Arc<(Condvar, Mutex<(bool, Option<T>)>)>);
impl<T> Sender<T> {
    pub fn send(self, v: T) {
        *self.0.1.lock().unwrap() = (true, Some(v));
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        self.0.1.lock().unwrap().0 = true;
        self.0.0.notify_one();
    }
}

#[derive(Debug)]
pub struct Receiver<T>(Arc<(Condvar, Mutex<(bool, Option<T>)>)>);

impl<T> Receiver<T> {
    pub fn recv(self) -> Option<T> {
        self.0
            .0
            .wait_while(self.0.1.lock().unwrap(), |x| !x.0)
            .unwrap()
            .1
            .take()
    }
}

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Arc::new((Condvar::new(), Mutex::new((false, None))));
    (Sender(inner.clone()), Receiver(inner))
}
