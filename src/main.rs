use std::sync::Arc;

mod socket;
mod tlv;
mod dispatch;

const NUM_DISPATCH_THREADS: i32 = 4;

fn main() {
    // Connection-to-dispatcher queue
    let (s1, r1) = crossbeam::channel::bounded::<Arc<socket::UdpPacket>>(50);

    // Start dispatch threads
    let mut dispatchers = Vec::new();
    for _ in 0..NUM_DISPATCH_THREADS {
        dispatchers.push(dispatch::thread(r1.clone()));
    }

    // Start listening for data
    socket::listen_udp("127.0.0.1:7766", s1).unwrap();
}
