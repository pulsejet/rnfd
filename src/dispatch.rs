use super::tlv;
use super::socket::UdpPacket;
use std::sync::Arc;

use crossbeam::deque::Injector;

// /8=localhost/8=nfd
const MGMT_MATCH: &'static [u8] = &[8, 9, 108, 111, 99, 97, 108, 104, 111, 115, 116, 8, 3, 110, 102, 100];

pub fn thread(
    chan_in: Arc<Injector<Arc<UdpPacket>>>,
    chan_mgmt: Arc<Injector<Arc<UdpPacket>>>,
    chans_out: Vec<Arc<Injector<Arc<UdpPacket>>>>,
) {
    std::thread::spawn(move || {
        loop {
            let steal = chan_in.steal();
            match steal {
                crossbeam::deque::Steal::Success(packet) => {
                    dispatch_udp(packet, &chan_mgmt, &chans_out);
                }
                crossbeam::deque::Steal::Empty => {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
                crossbeam::deque::Steal::Retry => {}
            }
        }
    });
}

fn dispatch_udp(
    packet: Arc<UdpPacket>,
    chan_mgmt: &Arc<Injector<Arc<UdpPacket>>>,
    chans_out: &Vec<Arc<Injector<Arc<UdpPacket>>>>,
) {
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

                // Check validity of name size
                let o = tlo.o+name_tlo.o;
                if o+name_tlo.l as usize > packet.data.len() {
                    return;
                }

                // Check if this is a management packet
                if name_tlo.l as usize >= MGMT_MATCH.len() {
                    let mut is_mgmt = true;
                    for i in 0..MGMT_MATCH.len() {
                        if packet.data[o..][i] != MGMT_MATCH[i] {
                            is_mgmt = false;
                            break;
                        }
                    }
                    if is_mgmt {
                        chan_mgmt.push(packet);
                        return;
                    }
                }

                if name_tlo.l > 30  { // hackkkkkkkk
                    // flood to all pipelines
                    for chan in chans_out {
                        chan.push(packet.clone());
                    }
                    return;
                }

                // Hash the name
                // TODO: drop segment number in this hashing
                let mut hash = 0;
                for b in &packet.data[o..o+name_tlo.l as usize] {
                    hash += *b as u64;
                }
                let idx = (hash % chans_out.len() as u64) as usize;
                chans_out[idx].push(packet);
            } else {
                println!("dispatch: unknown TLV type, dropping {:?}", tlo.t);
            }
        }
        Err(e) => {
            println!("Error decoding packet: {:?}", e);
        }
    }
}