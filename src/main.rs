use std::{sync::Arc, net::SocketAddr};

use crossbeam::deque::Injector;

mod socket;
mod tlv;
mod dispatch;
mod pipeline;
mod table;
mod mgmt;

const NUM_DISPATCH_THREADS: usize = 4;
const NUM_PIPELINE_THREADS: usize = 4;

fn main() {
    // Connection-to-dispatcher queue
    let q1 = Arc::new(Injector::<Arc<socket::UdpPacket>>::new());

    // Dispatcher-to-pipeline queues
    let mut pipeline_queues = Vec::new();
    for _ in 0..NUM_PIPELINE_THREADS {
        let q2 = Arc::new(Injector::<Arc<socket::UdpPacket>>::new());
        pipeline_queues.push(q2);
    }

    // Dispatcher-to-management queue
    let qm = Arc::new(Injector::<Arc<socket::UdpPacket>>::new());

    // Start dispatch threads
    let mut dispatchers = Vec::new();
    for i in 0..NUM_DISPATCH_THREADS {
        println!("Starting dispatcher thread {i}");

        // Clone pipeline queues for this thread
        let mut queues = Vec::new();
        for qs in &pipeline_queues {
            queues.push(qs.clone());
        }
        dispatchers.push(dispatch::thread(q1.clone(), qm.clone(), queues));
    }

    // Pipeline to connection queue
    let q3 = Arc::new(Injector::<(Vec<u8>, SocketAddr)>::new());

    { // Start management thread
        let mut queues = Vec::new();
        for qs in &pipeline_queues {
            queues.push(qs.clone());
        }
        mgmt::thread(qm.clone(), q3.clone(), queues);
    }

    // Start pipeline threads
    let mut pipelines = Vec::new();
    for i in 0..NUM_PIPELINE_THREADS {
        println!("Starting pipeline thread {i}");
        pipelines.push(pipeline::incoming::thread(pipeline_queues[i as usize].clone(), q3.clone()));
    }

    // Start listening for data
    println!("Starting UDP listener");
    socket::listen_udp("127.0.0.1:7766", q1.clone(), q3.clone()).unwrap();
    socket::listen_udp("127.0.0.1:7766", q1.clone(), q3.clone()).unwrap();
}
