use std::net::SocketAddr;
use std::sync::Arc;

use crossbeam::deque::Injector;

use crate::socket::UdpPacket;
use crate::tlv;
use crate::table::Table;

pub fn thread(chan_in: Arc<Injector<Arc<UdpPacket>>>, chan_out: Arc<Injector<(Vec<u8>, SocketAddr)>>) {
    std::thread::spawn(move || {
        let mut table = Table::new(chan_out);
        loop {
            let steal = chan_in.steal();
            match steal {
                crossbeam::deque::Steal::Success(packet) => {
                    process_packet(&mut table, packet);
                }
                crossbeam::deque::Steal::Empty => {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
                crossbeam::deque::Steal::Retry => {}
            }
        }
    });
}

fn process_packet(table: &mut Table, packet: Arc<UdpPacket>) {
    if packet.addr.port() == 0 {
        crate::mgmt::process_frame(table, packet);
        return;
    }

    let p_tlo = tlv::vec_decode::read_tlo(&packet.data[..]).unwrap(); // already checked
    if p_tlo.t == tlv::Type::Interest as u64 {
        super::interest::process_interest(table, packet, p_tlo);
    } else if p_tlo.t == tlv::Type::Data as u64 {
        super::data::process_data(table, packet, p_tlo);
    } else {
        println!("incoming: unknown TLV type, dropping: {:?}", p_tlo.t);
    }
}