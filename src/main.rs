use std::sync::Arc;

mod socket;
mod tlv;

const NUM_DISPATCH_THREADS: i32 = 4;

fn main() {
    // Connection-to-dispatcher queue
    let (s1, r1) = crossbeam::channel::bounded::<Arc<tlv::TLV>>(5);

    // Start dispatch threads
    let mut dispatchers = Vec::new();
    for _ in 0..NUM_DISPATCH_THREADS {
        let rc = r1.clone();
        dispatchers.push(std::thread::spawn(move || {
            loop {
                let packet = rc.recv().unwrap();
                println!("Dispatch: {:?}", packet);
            }
        }));
    }

    // Start listening for connections
    socket::listen_unix("/tmp/rnfd.sock", s1);
}
