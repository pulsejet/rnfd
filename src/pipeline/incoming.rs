use std::sync::Arc;
use crossbeam::channel::Receiver;

use crate::socket::UdpPacket;
use crate::tlv;
use crate::table::Table;

pub fn thread(chan_in: Receiver<Arc<UdpPacket>>) {
    std::thread::spawn(move || {
        let mut table = Table::new();
        loop {
            let packet = chan_in.recv().unwrap();
            process_packet(&mut table, packet);
        }
    });
}

fn process_packet(table: &mut Table, packet: Arc<UdpPacket>) {
    let p_tlo = tlv::vec_decode::read_tlo(&packet.data[..]).unwrap(); // already checked
    if p_tlo.t == tlv::Type::Interest as u64 {
        super::interest::process_interest(table, packet, p_tlo);
    } else {
        println!("Unknown TLV type, dropping");
    }
}