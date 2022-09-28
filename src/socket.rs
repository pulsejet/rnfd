use super::tlv;

use std::io::BufReader;
use std::os::unix::net::{UnixStream,UnixListener};
use std::path::Path;
use std::sync::Arc;
use crossbeam::channel::Sender;

fn handle_client(stream: UnixStream, sender: Sender<Arc<tlv::TLV>>) {
    let mut stream = BufReader::new(stream);
    loop {
        let res = tlv::stream_decode::read_tlv(&mut stream);
        match res {
            Ok(packet) => {
                println!("TLV: {:?}", packet);

                // Quickly hand off the TLV to a dispatch thread
                let res = sender.send(Arc::clone(&packet));
                if res.is_err() {
                    // Disconnected from dispatcher
                    // Drop the connection
                    break;
                }
                // if packet.t == tlv::Type::Interest as u64 {
                //     println!("Interest: {:?}", packet);
                // } else if packet.t == tlv::Type::Data as u64 {
                //     println!("Data: {:?}", packet);
                // }
            }
            Err(e) => {
                println!("DecodeError: {:?}", e);
                break;
            }
        }
    }
}

pub fn listen_unix(path: impl AsRef<Path>, sender: Sender<Arc<tlv::TLV>>) {
    let path = path.as_ref();
    let res = std::fs::remove_file(path);
    if res.is_err() {}

    let listener = UnixListener::bind(path).unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let s1 = sender.clone();
                std::thread::spawn(move || {
                    handle_client(stream, s1);
                });
            }
            Err(err) => {
                println!("Error: {}", err);
                break;
            }
        }
    }
}