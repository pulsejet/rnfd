use std::net::UdpSocket;
use std::sync::Arc;
use crossbeam::channel::Sender;

#[derive(Debug)]
pub struct UdpPacket {
    pub data: Vec<u8>,
    pub addr: std::net::SocketAddr,
}

pub fn listen_udp(path: &str, sender: Sender<Arc<UdpPacket>>) -> Result<(), std::io::Error> {
    let socket = UdpSocket::bind(path)?;

    let mut buf = [0; 8800];

    loop {
        let (amt, src) = socket.recv_from(&mut buf)?;
        let trun = &buf[..amt];
        let packet = Arc::new(UdpPacket {
            data: trun.to_vec(),
            addr: src,
        });
        if sender.send(packet).is_err() {
            println!("Failed to send packet to dispatcher");
        }
    }
}