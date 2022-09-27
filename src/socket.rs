use super::tlv;

use std::io::{BufReader, Read};
use std::os::unix::net::{UnixStream,UnixListener};
use std::thread;
use std::path::Path;

fn handle_client(stream: UnixStream) {
    let mut stream = BufReader::new(stream);
    let res = tlv::streamread::read_tlv(&mut stream);
    if (res.is_ok()) {
        println!("Got {:?}", res.unwrap());
    } else {
        println!("Error: {:?}", res.err().unwrap());
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