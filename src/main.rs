use std::{sync::Arc, net::SocketAddr};

mod socket;
mod tlv;
mod dispatch;
mod pipeline;
mod table;
mod mgmt;

const NUM_DISPATCH_THREADS: usize = 2;
const NUM_PIPELINE_THREADS: usize = 2;

fn main() {
    // Connection-to-dispatcher queue
    let (s1, r1) = crossbeam::channel::bounded::<Arc<socket::UdpPacket>>(50);

    // Dispatcher-to-pipeline queues
    let mut pipeline_queues = Vec::new();
    for _ in 0..NUM_PIPELINE_THREADS {
        let (s2, r2) = crossbeam::channel::bounded::<Arc<socket::UdpPacket>>(50);
        pipeline_queues.push((s2, r2));
    }

    // Dispatcher-to-management queue
    let (sm, rm) = crossbeam::channel::bounded::<Arc<socket::UdpPacket>>(50);

    // Start dispatch threads
    let mut dispatchers = Vec::new();
    for i in 0..NUM_DISPATCH_THREADS {
        println!("Starting dispatcher thread {i}");

        // Clone pipeline queues for this thread
        let mut queues = Vec::new();
        for (s, _) in &pipeline_queues {
            queues.push(s.clone());
        }
        dispatchers.push(dispatch::thread(r1.clone(), sm.clone(), queues));
    }

    // Pipeline to connection queue
    let (s3, r3) = crossbeam::channel::bounded::<(Vec<u8>, SocketAddr)>(50);

    { // Start management thread
        let mut queues = Vec::new();
        for (s, _) in &pipeline_queues {
            queues.push(s.clone());
        }
        mgmt::thread(rm.clone(), s3.clone(), queues);
    }

    // Start pipeline threads
    let mut pipelines = Vec::new();
    for i in 0..NUM_PIPELINE_THREADS {
        println!("Starting pipeline thread {i}");
        pipelines.push(pipeline::incoming::thread(pipeline_queues[i as usize].1.clone(), s3.clone()));
    }

    // Start listening for data
    println!("Starting UDP listener");
    socket::listen_udp("127.0.0.1:7766", s1, r3).unwrap();
}
