use super::tlv::TLV;
use std::sync::Arc;

use crossbeam::channel::Receiver;

pub fn thread(chan_in: Receiver<Arc<TLV>>) {
    std::thread::spawn(move || {
        loop {
            let packet = chan_in.recv().unwrap();
            println!("Dispatch: {:?}", packet);
        }
    });
}