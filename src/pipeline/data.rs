use std::sync::Arc;

use crate::{table::Table, socket::UdpPacket, tlv};

pub fn process_data(table: &mut Table, packet: Arc<UdpPacket>, p_tlo: tlv::TLO) {
    // Get name
    let name_tlo = tlv::vec_decode::read_tlo(&packet.data[p_tlo.o..]).unwrap(); // already checked
    let name = &packet.data[p_tlo.o+name_tlo.o..p_tlo.o+name_tlo.o+name_tlo.l as usize];

    // Get PIT entry
    let entries = table.pit.get_all_can_be_pfx(&name.to_vec());
    if entries.len() == 0 {
        println!("No PIT entry for data, dropping: {:?}", name);
        return;
    }

    // Send to all downstreams for all inrecords
    for entry in entries {
        for in_record in &entry.borrow().in_records {
            table.send_chan.push((packet.data.clone(), in_record.face));
        }
    }
}