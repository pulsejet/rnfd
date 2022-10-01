use std::net::SocketAddr;
use std::mem::MaybeUninit;
use std::io::{BufReader, IoSlice, IoSliceMut};
use std::os::unix::net::{UnixStream,UnixListener};
use std::os::unix::prelude::AsRawFd;
use std::sync::Arc;
use std::io::Write;

use nix::sys::socket::{MsgFlags, RecvMmsgData, RecvMsg, SockaddrIn};
mod stream_decode;

fn handle_client(stream: UnixStream) {
    // Start UDP socket to rNFD
    let socket = socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::DGRAM, None).unwrap();
    socket.set_recv_buffer_size(10000 * 2000).unwrap();
    socket.set_send_buffer_size(10000 * 2000).unwrap();
    let socket_arc = Arc::new(socket);
    let socket_arc_clone = socket_arc.clone();

    // Stream arc
    let stream_arc = Arc::new(stream);
    let stream_arc_clone = stream_arc.clone();

    // Start thread to read from UDP socket and write to Unix socket
    std::thread::spawn(move || {
        let mut stream = &*stream_arc_clone;
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
                    nix::sys::socket::recvmmsg(socket_arc_clone.as_raw_fd(), &mut msgs, MsgFlags::MSG_DONTWAIT, None);
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

            let mut data = Vec::new();
            for i in 0..receive_buffers_bytes.len() {
                if receive_buffers_bytes[i] == 0 {
                    continue;
                }

                data.extend_from_slice(&receive_buffers[i][..receive_buffers_bytes[i]]);
            }

            let res = stream.write(&data);
            if res.is_err() {
                println!("Error writing to Unix socket");
                return;
            }
        }
    });

    // Start thread to read from unix socket and write to UDP socket
    stream_arc.set_read_timeout(Some(std::time::Duration::from_millis(1))).unwrap();
    let mut stream = BufReader::with_capacity(8800*20, &*stream_arc);
    let addr: SocketAddr = "127.0.0.1:7766".parse().unwrap();

    let mut datas = vec![];

    loop {
        let mut should_send = false;
        let res = stream_decode::read_tlv(&mut stream);
        match res {
            Ok(packet) => {
                datas.push(packet.data);

                if datas.len() >= 10 {
                    should_send = true;
                }
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    should_send = true;
                } else {
                    println!("Error reading from Unix socket: {:?}", e);
                    return;
                }
            }
        }

        if should_send {
            if datas.len() == 0 {
                continue;
            }

            let mut iovs = vec![];
            let mut msgs = vec![];

            for i in 0..datas.len() {
                let iov = [IoSlice::new(&datas[i])];
                iovs.push(iov);
            }

            for i in 0..iovs.len() {
                let s_addr;
                match addr {
                    SocketAddr::V4(v4addr) => {
                        s_addr = nix::sys::socket::SockaddrIn::from(v4addr);
                    },
                    SocketAddr::V6(_) => {
                        println!("IPv6 not supported yet");
                        continue;
                    }
                }
                msgs.push(nix::sys::socket::SendMmsgData {
                    iov: &iovs[i],
                    cmsgs: &[],
                    addr: Some(s_addr),
                    _lt: Default::default(),
                });
            }

            let res = nix::sys::socket::sendmmsg(socket_arc.clone().as_raw_fd(), &msgs, MsgFlags::empty());
            match res {
                Ok(_) => {}
                Err(e) => {
                    println!("Error: {:?}", e);
                }
            }
            datas = vec![];
        }
    }
}

fn main() {
    let path: String = "/tmp/rnfd.sock".to_string();
    let res = std::fs::remove_file(&path);
    if res.is_err() {}

    let listener = UnixListener::bind(&path).unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(move || {
                    handle_client(stream);
                });
            }
            Err(err) => {
                println!("Error: {}", err);
                break;
            }
        }
    }
}