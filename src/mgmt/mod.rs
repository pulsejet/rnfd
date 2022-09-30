pub mod table;

use std::{sync::Arc, net::SocketAddr};
use crossbeam::channel::{Receiver, Sender};
use crate::socket::UdpPacket;

pub fn thread(
    chan_in: Receiver<Arc<UdpPacket>>,
    chan_out: Sender::<(Vec<u8>, SocketAddr)>,
    chans_pipeline: Vec<Sender<Arc<UdpPacket>>>,
) {
    std::thread::spawn(move || {
        loop {
            let packet = chan_in.recv().unwrap();
            process_mgmt(packet, &chan_out, &chans_pipeline);
        }
    });
}

fn process_mgmt(
    packet: Arc<UdpPacket>,
    chan_out: &Sender<(Vec<u8>, SocketAddr)>,
    chans_pipeline: &Vec<Sender<Arc<UdpPacket>>>,
) {
    println!("Got a management packet, {:?}", packet.data);
}