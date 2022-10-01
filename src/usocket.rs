use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::io::{Write, BufReader};
use std::os::unix::net::UnixStream;
use std::{mem::MaybeUninit, os::unix::net::UnixListener};
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use crossbeam::channel::{Sender, Receiver};

use crate::socket::UdpPacket;
use crate::unix_socket::stream_decode;

pub fn listen_udp(
    path: &str,
    sender: Sender<Arc<UdpPacket>>,
    receiver: Receiver<(Vec<u8>, SocketAddr)>
) -> Result<(), std::io::Error> {
    let path: String = "/tmp/rnfd.sock".to_string();
    let res = std::fs::remove_file(&path);
    if res.is_err() {}

    let listener = UnixListener::bind(&path).unwrap();

    let socket_map: HashMap<u16, Arc<UnixStream>> = std::collections::HashMap::new();
    let socket_map = Arc::new(RwLock::new(socket_map));
    let sender = Arc::new(sender);

    thread_out(socket_map.clone(), receiver);

    let mut count = 1;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let socket = Arc::new(stream);
                socket_map.write().unwrap().insert(count, socket.clone());
                thread_in(socket.clone(), sender.clone(), count);
                count += 1;
            }
            Err(err) => {
                println!("Error: {}", err);
                break;
            }
        }
    }
    Ok(())
}

fn thread_out(table: Arc<RwLock<HashMap<u16, Arc<UnixStream>>>>, receiver: Receiver<(Vec<u8>, SocketAddr)>) {
    std::thread::spawn(move || {
        loop {
            let (data, addr) = receiver.recv().unwrap();
            let t = table.read().unwrap();
            let stream = t.get(&addr.port());
            if stream.is_none() {
                continue;
            }
            let mut stream = &**stream.unwrap();
            let res = stream.write(&data);
            if res.is_err() {
                println!("Error writing to Unix socket");
                table.write().unwrap().remove(&addr.port());
                return;
            }
        }
    });
}

fn thread_in(stream_arc: Arc<UnixStream>, sender: Arc<Sender<Arc<UdpPacket>>>, fake_port: u16) {
    std::thread::spawn(move || {
        let mut stream = BufReader::new(&*stream_arc);
        loop {
            let res = stream_decode::read_tlv(&mut stream);
            match res {
                Ok(packet) => {
                    let packet = Arc::new(UdpPacket {
                        data: packet.data,
                        addr: SocketAddr::from(([0, 0, 0, 0], fake_port)),
                    });
                    sender.send(packet).unwrap();
                }
                Err(e) => {
                    println!("DecodeError: {:?}", e);
                    break;
                }
            }
        }
    });
}