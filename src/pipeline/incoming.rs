use std::sync::Arc;
use crossbeam::channel::Receiver;

use crate::pipeline::Interest;
use crate::socket::UdpPacket;
use crate::tlv;
use crate::table::Table;

pub fn thread(chan_in: Receiver<Arc<UdpPacket>>) {
    std::thread::spawn(move || {
        let table = Table::new();
        loop {
            let packet = chan_in.recv().unwrap();
            process_packet(&table, packet);
        }
    });
}

fn process_packet(table: &Table, packet: Arc<UdpPacket>) {
    let p_tlo = tlv::vec_decode::read_tlo(&packet.data[..]).unwrap(); // already checked
    if p_tlo.t == tlv::Type::Interest as u64 {
        process_interest(table, packet, p_tlo);
    } else {
        println!("Unknown TLV type, dropping");
    }
}

fn process_interest(table: &Table, packet: Arc<UdpPacket>, p_tlo: tlv::TLO) {
    // Get name
    let name_tlo = tlv::vec_decode::read_tlo(&packet.data[p_tlo.o..]).unwrap(); // already checked
    let name = &packet.data[p_tlo.o+name_tlo.o..p_tlo.o+name_tlo.o+name_tlo.l as usize];

    // Make Interest struct
    let mut interest = Interest::new(name.to_vec());

    // Get Interest parameters
    // TODO: forwarding hint
    let mut o = p_tlo.o + name_tlo.o + name_tlo.l as usize;
    while o < p_tlo.o + p_tlo.l as usize {
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

    // Insert into PIT
}