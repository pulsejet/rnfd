use std::net::SocketAddr;
use std::sync::Arc;
use crossbeam::channel::{Receiver, Sender};

use crate::socket::UdpPacket;
use crate::tlv;
use crate::table::Table;

pub fn thread(chan_in: Receiver<Arc<UdpPacket>>, chan_out: Sender<(Vec<u8>, SocketAddr)>) {
    std::thread::spawn(move || {
        let mut table = Table::new(chan_out);

        // Create dummy FIB
        let name = vec![8, 3, 110, 100, 110, 8, 5, 118, 97, 114, 117, 110]; // /ndn/varun
        let hop = crate::table::pit::NextHop {
            addr: std::net::SocketAddr::V4("127.0.0.1:8000".parse().unwrap()),
            cost: 1,
        };
        let e1 = table.pit.insert_or_get(&name);
        if let Ok((e, _, _)) = e1 {
            e.borrow_mut().nexthops.push(hop.clone());
        }

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