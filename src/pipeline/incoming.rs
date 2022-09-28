use std::sync::Arc;
use crossbeam::channel::Receiver;
use super::super::socket::UdpPacket;

pub fn thread(chan_in: Receiver<Arc<UdpPacket>>) {
    std::thread::spawn(move || {
        loop {
            let packet = chan_in.recv().unwrap();
            println!("Got incoming packet: {:?}", packet);
        }
    });
}