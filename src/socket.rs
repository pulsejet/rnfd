use super::tlv;

use std::io::BufReader;
use std::os::unix::net::{UnixStream,UnixListener};
use std::thread;
use std::path::Path;

fn handle_client(stream: UnixStream) {
    let mut stream = BufReader::new(stream);
    loop {
        let res = tlv::streamread::read_tlv(&mut stream);
        match res {
            Ok(tlv) => {
                println!("TLV: {:?}", tlv);
            }
            Err(e) => {
                println!("DecodeError: {:?}", e);
                break;
            }
        }
    }
}

pub fn listen_unix(path: impl AsRef<Path>) {
    let path = path.as_ref();
    let res = std::fs::remove_file(path);
    if res.is_err() {}

    let listener = UnixListener::bind(path).unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| handle_client(stream));
            }
            Err(err) => {
                println!("Error: {}", err);
                break;
            }
        }
    }
}