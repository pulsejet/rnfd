use std::net::SocketAddr;
use std::mem::MaybeUninit;
use std::io::BufReader;
use std::os::unix::net::{UnixStream,UnixListener};
use std::sync::Arc;
use std::io::Write;
mod stream_decode;

fn handle_client(mut stream: UnixStream) {
    // Start UDP socket to rNFD
    let socket = socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::DGRAM, None).unwrap();
    let socket_arc = Arc::new(socket);
    let socket_arc_clone = socket_arc.clone();

    // Stream arc
    let stream_arc = Arc::new(stream);
    let stream_arc_clone = stream_arc.clone();

    // Start thread to read from UDP socket and write to Unix socket
    let t = std::thread::spawn(move || {
        let mut stream = &*stream_arc_clone;
        let mut buf: [MaybeUninit<u8>; 8800] = unsafe {
            MaybeUninit::uninit().assume_init()
        };

        loop {
            let res = socket_arc_clone.recv_from(&mut buf);
            match res {
                Ok((amt, src)) => {
                    let data = unsafe { std::slice::from_raw_parts(buf.as_ptr() as *const u8, amt) };
                    let res = stream.write(&data);
                    if res.is_err() {
                        println!("Error writing to Unix socket");
                        return;
                    }
                }
                Err(_) => {}
            }
        }
    });

    // Start thread to read from unix socket and write to UDP socket
    let mut stream = BufReader::new(&*stream_arc);
    loop {
        let res = stream_decode::read_tlv(&mut stream);
        match res {
            Ok(packet) => {
                let addr: SocketAddr = "127.0.0.1:7766".parse().unwrap();
                socket_arc.send_to(&packet.data, &addr.into()).unwrap();
            }
            Err(e) => {
                println!("DecodeError: {:?}", e);
                break;
            }
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