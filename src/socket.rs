use std::mem::MaybeUninit;
use std::net::SocketAddr;
use std::sync::Arc;
use crossbeam::deque::Injector;
use socket2::Socket;

#[derive(Debug)]
pub struct UdpPacket {
    pub data: Vec<u8>,
    pub addr: std::net::SocketAddr,
}

pub fn listen_udp(
    path: &str,
    sender: Arc<Injector<Arc<UdpPacket>>>,
    receiver: Arc<Injector<(Vec<u8>, SocketAddr)>>
) -> Result<(), std::io::Error> {
    let addr: SocketAddr = path.parse().unwrap();
    let socket = socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::DGRAM, None).unwrap();
    socket.set_reuse_address(true).unwrap();
    socket.bind(&addr.into()).unwrap();
    let socket_arc = Arc::new(socket);

    thread_out(socket_arc.clone(), receiver.clone());
    thread_in(socket_arc.clone(), sender.clone());
    Ok(())
}

fn thread_out(socket: Arc<Socket>, receiver: Arc<Injector<(Vec<u8>, SocketAddr)>>) {
    std::thread::spawn(move || {
        loop {
            let steal = receiver.steal();
            match steal {
                crossbeam::deque::Steal::Success(packet) => {
                    let (data, addr) = packet;
                    socket.send_to(&data, &addr.into()).unwrap();
                }
                crossbeam::deque::Steal::Empty => {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
                crossbeam::deque::Steal::Retry => {}
            }
        }
    });
}

fn thread_in(socket: Arc<Socket>, sender: Arc<Injector<Arc<UdpPacket>>>,) {
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
                sender.push(packet);
            }
            Err(_) => {}
        }
    }
}