use owned_mut_ref::OwnedMutRef;
use std::sync::mpsc;

fn main() {
    let mut x = 1;
    let mut_ref = &mut x;

    let (owned_mut, waiter) = OwnedMutRef::new(mut_ref);

    let (tx, rx) = mpsc::channel();
    tx.send(owned_mut).unwrap();
    let mut received_owned_mut = rx.recv().unwrap();
    *received_owned_mut += 1;

    drop(received_owned_mut); // allow the waiter to continue
    waiter.wait(); // must wait before reusing the original value
    // for async code, use `waiter.await`

    println!("Value is now {}", x);
}
