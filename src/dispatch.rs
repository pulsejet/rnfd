use super::tlv;
use super::socket::UdpPacket;
use std::sync::Arc;

use crossbeam::channel::Receiver;

pub fn thread(chan_in: Receiver<Arc<UdpPacket>>) {
    std::thread::spawn(move || {
        loop {
            let packet = chan_in.recv().unwrap();
            dispatch_udp(packet);
        }
    });
}

fn dispatch_udp(packet: Arc<UdpPacket>) {
    let mut stream = std::io::Cursor::new(&packet.data);
    let res = tlv::stream_decode::read_tlv(&mut stream);
    match res {
        Ok(packet) => {
            println!("Got packet: {:?}", packet);
        }
        Err(e) => {
            println!("Error decoding packet: {:?}", e);
        }
    }
}