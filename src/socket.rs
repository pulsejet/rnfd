use std::mem::MaybeUninit;
use std::net::SocketAddr;
use std::sync::Arc;
use crossbeam::channel::{Sender, Receiver};
use socket2::Socket;

#[derive(Debug)]
pub struct UdpPacket {
    pub data: Vec<u8>,
    pub addr: std::net::SocketAddr,
}

pub fn listen_udp(
    path: &str,
    sender: Sender<Arc<UdpPacket>>,
    receiver: Receiver<(Vec<u8>, SocketAddr)>
) -> Result<(), std::io::Error> {
    let addr: SocketAddr = path.parse().unwrap();
    let socket = socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::DGRAM, None).unwrap();
    socket.set_reuse_address(true).unwrap();
    socket.bind(&addr.into()).unwrap();
    let socket_arc = Arc::new(socket);

    thread_out(socket_arc.clone(), receiver);
    thread_in(socket_arc, sender);
    Ok(())
}

fn thread_out(socket: Arc<Socket>, receiver: Receiver<(Vec<u8>, SocketAddr)>) {
    std::thread::spawn(move || {
        loop {
            let (data, addr) = receiver.recv().unwrap();
            socket.send_to(&data, &addr.into()).unwrap();
        }
    });
}

fn thread_in(socket: Arc<Socket>, sender: Sender<Arc<UdpPacket>>,) {
    let mut buf: [MaybeUninit<u8>; 8800] = unsafe {
        MaybeUninit::uninit().assume_init()
    };

    loop {
        let res = socket.recv_from(buf.as_mut());
        match res {
            Ok((amt, src)) => {
                let data = unsafe { std::slice::from_raw_parts(buf.as_ptr() as *const u8, amt) };
                let packet = Arc::new(UdpPacket {
                    data: data.to_vec(),
                    addr: src.as_socket().unwrap(),
                });
                sender.send(packet).unwrap();
            }
            Err(_) => {}
        }
    }
}