use owned_mut_ref::OwnedMutRef;
use std::sync::mpsc;

fn main() {
    // create a mutable reference
    let mut x = 1;
    let mut_ref = &mut x;

    // create the owned_mut_ref and the owned_mut_ref_waiter
    let (owned_mut, waiter) = OwnedMutRef::new(mut_ref);

    // now do wathever you want with your owned_mut_ref
    // like send it through a channel
    let (tx, rx) = mpsc::channel();
    tx.send(owned_mut).unwrap();
    let mut received_owned_mut = rx.recv().unwrap();
    // then use it on the other end
    *received_owned_mut += 1;

    // dropping the received_owned_mut will allow the waiter to continue
    drop(received_owned_mut);

    // you must wait on the waiter
    // dropping the waiter will abort the program
    waiter.wait();

    // once you called wait the value is free to be used again
    println!("{x}");
}
