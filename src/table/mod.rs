use std::net::SocketAddr;
use crossbeam::channel::Sender;

use self::dnl::DeadNonceList;
use self::pit::PIT;

pub mod dnl;
pub mod pit;

const DNL_MAX_LENGTH: usize = 4096;

pub struct Table {
    pub dnl: DeadNonceList,
    pub pit: PIT,
    pub send_chan: Sender<(Vec<u8>, SocketAddr)>,
}

impl Table {
    pub fn new(send_chan: Sender<(Vec<u8>, SocketAddr)>) -> Table {
        Table {
            dnl: DeadNonceList::new(DNL_MAX_LENGTH),
            pit: PIT::new(),
            send_chan,
        }
    }

    pub fn clean(&mut self) {
        self.dnl.clean();
    }
}
