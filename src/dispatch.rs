use super::tlv;
use super::socket::UdpPacket;
use std::sync::Arc;
use crossbeam::channel::Sender;

use crossbeam::channel::Receiver;

pub fn thread(chan_in: Receiver<Arc<UdpPacket>>, chans_out: Vec<Sender<Arc<UdpPacket>>>) {
    std::thread::spawn(move || {
        loop {
            let packet = chan_in.recv().unwrap();
            dispatch_udp(packet, &chans_out);
        }
    });
}

fn dispatch_udp(packet: Arc<UdpPacket>, chans_out: &Vec<Sender<Arc<UdpPacket>>>) {
    let res = tlv::vec_decode::read_tlo(&packet.data[..]);
    match res {
        Ok(tlo) => {
            if tlo.t == tlv::Type::Interest as u64 || tlo.t == tlv::Type::Data as u64 {
                // Read the first TLV inside the packet
                // This is the name of the Interest or Data
                let res = tlv::vec_decode::read_tlo(&packet.data[tlo.o..]);
                if res.is_err() {
                    println!("Failed to read name TLV");
                    return;
                }
                let name_tlo = res.unwrap();
                if name_tlo.t != tlv::Type::Name as u64 {
                    println!("First TLV is not a Name");
                    return;
                }

                // Hash the name
                // TODO: drop segment number in this hashing
                let mut hash = 0;
                let o = tlo.o+name_tlo.o;
                for b in &packet.data[o..o+name_tlo.l as usize] {
                    hash += *b as u64;
                }
                let idx = (hash % chans_out.len() as u64) as usize;
                if chans_out[idx].send(packet).is_err() {
                    println!("Failed to send packet to pipeline");
                }
            } else {
                println!("Unknown TLV type, dropping");
            }
        }
        Err(e) => {
            println!("Error decoding packet: {:?}", e);
        }
    }
}