use std::sync::mpsc;
use sync_mut_ref::SyncMutRef;

fn main() {
    // create a mutable reference
    let mut x = 1;
    let mut_ref = &mut x;

    // create the sync_mut_ref and the sync_mut_ref waiter
    let (send_mut, waiter) = SyncMutRef::new(mut_ref);

    // now do wathever you want with your sync_mut_ref
    // like send it through a channel
    let (tx, rx) = mpsc::channel();
    tx.send(send_mut).unwrap();
    let mut received_sync_mut = rx.recv().unwrap();
    // then use it on the other end
    *received_sync_mut += 1;

    // dropping the received_sync_mut will allow the waiter to continue
    drop(received_sync_mut);

    // you must wait on the waiter
    // dropping the waiter will abort the program
    waiter.wait();

    // once you called wait the value is free to be used again
    println!("{x}");
}
