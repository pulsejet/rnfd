use std::io::{IoSlice, IoSliceMut};
use std::{mem::MaybeUninit};
use std::net::{SocketAddr, SocketAddrV4};
use std::os::unix::prelude::AsRawFd;
use std::sync::Arc;
use crossbeam::deque::{Injector, Worker};
use nix::sys::socket::{MsgFlags, RecvMmsgData, SockaddrIn, RecvMsg};
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
    println!("Starting UDP listener");

    let addr: SocketAddr = path.parse().unwrap();
    let socket = socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::DGRAM, None).unwrap();
    socket.set_recv_buffer_size(10000 * 2000).unwrap();
    socket.set_send_buffer_size(10000 * 2000).unwrap();
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
            let w = Worker::new_fifo();
            let steal = receiver.steal_batch(&w);
            match steal {
                crossbeam::deque::Steal::Success(_) => {
                    let mut msgs = vec![];
                    let mut datas = vec![];
                    let mut iovs = vec![];

                    while let Some(v) = w.pop() {
                        datas.push(v);
                    }

                    for i in 0..datas.len() {
                        let iov = [IoSlice::new(&datas[i].0)];
                        iovs.push(iov);
                    }

                    for i in 0..iovs.len() {
                        let pack_addr = datas[i].1;
                        let addr;
                        match pack_addr {
                            SocketAddr::V4(v4addr) => {
                                addr = nix::sys::socket::SockaddrIn::from(v4addr);
                            },
                            SocketAddr::V6(_) => {
                                println!("IPv6 not supported yet");
                                continue;
                            }
                        }
                        msgs.push(nix::sys::socket::SendMmsgData {
                            iov: &iovs[i],
                            cmsgs: &[],
                            addr: Some(addr),
                            _lt: Default::default(),
                        });
                    }

                    let res = nix::sys::socket::sendmmsg(socket.as_raw_fd(), &msgs, MsgFlags::empty());
                    match res {
                        Ok(_) => {}
                        Err(e) => {
                            println!("Error: {:?}", e);
                        }
                    }
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
    std::thread::spawn(move || {
        let mut receive_buffers = [[0u8; 2000]; 100];
        let mut receive_buffers_addrs = [MaybeUninit::uninit(); 100];
        let mut receive_buffers_bytes = [0usize; 100];

        loop {
            { // Receive data from UDP socket
                let iovs: Vec<_> = receive_buffers
                        .iter_mut()
                        .map(|buf| [IoSliceMut::new(&mut buf[..])])
                        .collect();
                let mut msgs = Vec::new();
                for iov in &iovs {
                    msgs.push(RecvMmsgData {
                        iov,
                        cmsg_buffer: None,
                    })
                }

                let res: Result<Vec<RecvMsg<SockaddrIn>>, nix::errno::Errno>=
                    nix::sys::socket::recvmmsg(socket.as_raw_fd(), &mut msgs, MsgFlags::MSG_DONTWAIT, None);
                match res {
                    Ok(vc) => {
                        for i in 0..receive_buffers_bytes.len() {
                            receive_buffers_bytes[i] = 0;
                        }

                        for i in 0..vc.len() {
                            let rr = vc[i];
                            if rr.address.is_none() {
                                continue;
                            }
                            let addr = rr.address.unwrap();
                            receive_buffers_addrs[i] = MaybeUninit::new(addr);
                            receive_buffers_bytes[i] = rr.bytes;
                        }
                    }
                    Err(e) => {
                        match e {
                            nix::errno::Errno::EAGAIN => {
                                std::thread::sleep(std::time::Duration::from_millis(1));
                            }
                            _ => {
                                println!("Error: {:?}", e);
                                return;
                            }
                        }
                        continue;
                    }
                }
            }

            // Write data to queue
            for i in 0..receive_buffers_bytes.len() {
                if receive_buffers_bytes[i] == 0 {
                    continue;
                }

                let data = &receive_buffers[i][..receive_buffers_bytes[i]];

                let addr = unsafe { receive_buffers_addrs[i].assume_init() };
                let packet = Arc::new(UdpPacket {
                    data: data.to_vec(),
                    addr: SocketAddrV4::new(addr.ip().into(), addr.port()).into(),
                });

                sender.push(packet);
            }
        }
    });
}