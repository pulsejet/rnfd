use std::sync::Arc;

use crate::pipeline::Interest;
use crate::socket::UdpPacket;
use crate::table::pit::PITEntry;
use crate::tlv;
use crate::table::Table;

pub fn process_interest(table: &mut Table, packet: Arc<UdpPacket>, p_tlo: tlv::TLO) {
    // Get name
    let name_tlo = tlv::vec_decode::read_tlo(&packet.data[p_tlo.o..]).unwrap(); // already checked
    let name = &packet.data[p_tlo.o+name_tlo.o..p_tlo.o+name_tlo.o+name_tlo.l as usize];

    // Make Interest struct
    let mut interest = Interest::new(name.to_vec(), p_tlo);

    // Get Interest parameters
    // TODO: forwarding hint
    let mut o = interest.outer_tlo.o + name_tlo.o + name_tlo.l as usize;
    while o < interest.outer_tlo.o + interest.outer_tlo.l as usize {
        let res = tlv::vec_decode::read_tlo(&packet.data[o..]);
        if res.is_err() {
            println!("Failed to read TLV");
            return;
        }
        let tlo = res.unwrap();
        let tt: tlv::Type = unsafe { ::std::mem::transmute(tlo.t) };
        match tt {
            tlv::Type::CanBePrefix => {
                interest.can_be_prefix = Some(true);
            }
            tlv::Type::MustBeFresh => {
                interest.must_be_fresh = Some(true);
            }
            tlv::Type::Nonce => {
                let nonce = tlv::vec_decode::read_u32(&packet.data[o+tlo.o..]).unwrap();
                interest.nonce = Some(nonce);
            }
            tlv::Type::InterestLifetime => {
                let lifetime = tlv::vec_decode::read_nni(&packet.data[o+tlo.o..], tlo.l).unwrap();
                interest.lifetime = Some(lifetime);
            }
            tlv::Type::HopLimit => {
                let hop_limit = tlv::vec_decode::read_u8(&packet.data[o+tlo.o..]).unwrap();
                interest.hop_limit = Some(hop_limit);
            }
            _ => {
                // TODO: evolvability
            }
        }
        o += tlo.o + tlo.l as usize;
    }

    // Log
    println!("Incoming {:?}", interest);

    // Check hop limit
    match interest.hop_limit {
        Some(hop_limit) => { if hop_limit == 0 { return; } }
        None => {}
    }
    // TODO: decrement hop limit

    // TODO: localhost scope violation check

    // Get 64-bit nonce hash and check against dead nonce list
    let nonce = match interest.nonce {
        Some(nonce) => nonce,
        None => { return; } // we don't forward interests without a nonce
    };
    let nonce_hash = fasthash::metro::hash64_with_seed(&name[..], nonce);
    if table.dnl.contains(nonce_hash) {
        // TODO: onInterestLoop (send NACK)
        return;
    }

    // Insert into DNL
    // This behavior is different from NFD; rNFD does not use PIT
    // to check for duplicate nonce
    table.dnl.insert(nonce_hash);

    // Insert into PIT, this will get aggregated automatically
    // TODO: forwarding hint checks
    let entry = table.pit.insert_or_get(&interest.name);
    match entry {
        Ok(entry) => {
            let is_new = entry.pending_interests.len() == 0;
            entry.pending_interests.push_back(PITEntry::new(&interest, packet.addr));

            if is_new {
                // look up content store
                on_cs_miss(table, packet, interest);
            } else {
                on_cs_miss(table, packet, interest);
            }
        }
        Err(_) => {}
    }
}

fn on_cs_miss(table: &mut Table, packet: Arc<UdpPacket>, interest: Interest) {
    // TODO: set PIT expiry timer to the time that the last PIT in-record expires

    println!("CS miss for {:?}", interest);
}