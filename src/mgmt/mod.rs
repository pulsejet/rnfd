pub mod table;

use std::io::Read;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::{sync::Arc, net::SocketAddr};
use crossbeam::channel::{Receiver, Sender};
use crate::socket::UdpPacket;
use crate::tlv;
use crate::tlv::varnumber::VarNumber;

pub fn thread(
    chan_in: Receiver<Arc<UdpPacket>>,
    chan_out: Sender::<(Vec<u8>, SocketAddr)>,
    chans_pipeline: Vec<Sender<Arc<UdpPacket>>>,
) {
    // Connect to YaNFD socket
    let stream = UnixStream::connect("/tmp/yanfd.sock.rnfd").unwrap();
    let stream_arc = Arc::new(stream);
    let stream_arc_read = stream_arc.clone();

    // Read from input channel and write to YaNFD
    std::thread::spawn(move || {
        send_yanfd(chan_in, stream_arc);
    });

    // Read from YaNFD socket
    std::thread::spawn(move || {
        read_yanfd(stream_arc_read, chan_out)
    });
}

fn read_yanfd(stream_arc: Arc<UnixStream>, chan_out: Sender::<(Vec<u8>, SocketAddr)>) {
    loop {
        let mut buf = [0; 8800];
        let mut stream = &*stream_arc;
        let len = stream.read(&mut buf).unwrap();
        if len == 0 {
            panic!("YaNFD socket closed");
        }

        let frame = &buf[..len];
        let res = read_yanfd_frame(frame);
        match res {
            Ok((addr, data)) => {
                println!("YaNFD: read {} bytes for {:?}", data.len(), addr);
                chan_out.send((data, addr)).unwrap();
            },
            Err(e) => {
                println!("YaNFD: parsing error {:?}", e);
            }
        }
    }
}

fn read_yanfd_frame(frame: &[u8]) -> Result<(SocketAddr, Vec<u8>), std::io::Error> {
    let frame_tlo = tlv::vec_decode::read_tlo(frame)?;

    let c_frame = &frame[frame_tlo.o..];

    // Get address (TLV type 4)
    let addr_tlo = tlv::vec_decode::read_tlo(c_frame)?;
    if addr_tlo.t != 4 {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Expected TLV type 4"));
    }
    let addr = &c_frame[addr_tlo.o..addr_tlo.o+addr_tlo.l as usize];
    let addr_str = std::str::from_utf8(addr);
    if addr_str.is_err() {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Invalid address"));
    }
    let addr = addr_str.unwrap().parse::<SocketAddr>();
    if addr.is_err() {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Invalid address"));
    }
    let addr = addr.unwrap();

    // Peel off link layer header
    let mut data = &c_frame[addr_tlo.o+addr_tlo.l as usize..];
    let mut data_tlo = tlv::vec_decode::read_tlo(data)?;
    while data_tlo.t != 6 {
        data = &data[data_tlo.o as usize..];
        data_tlo = tlv::vec_decode::read_tlo(data)?;
    }

    Ok((addr, data.to_vec()))
}

fn send_yanfd(chan_in: Receiver<Arc<UdpPacket>>, stream_arc: Arc<UnixStream>) {
    loop {
        let packet = chan_in.recv().unwrap();
        println!("Got a management packet, from {}", packet.addr.to_string());
        let mut stream = &*stream_arc;

        let addr_str = packet.addr.to_string();
        let addr_bytes = addr_str.as_bytes();
        let addr_len = addr_bytes.len() as usize;

        let mut addr_vec = Vec::new();
        addr_vec.extend_from_slice(&VarNumber::from(4 as u64).to_bytes());
        addr_vec.extend_from_slice(&VarNumber::from(addr_len as u64).to_bytes());
        addr_vec.extend_from_slice(&addr_bytes);

        let mut data_vec = Vec::new();
        data_vec.extend_from_slice(&VarNumber::from(21 as u64).to_bytes());
        data_vec.extend_from_slice(&VarNumber::from(packet.data.len() as u64).to_bytes());
        data_vec.extend_from_slice(&packet.data);

        let mut outer_vec = Vec::new();
        outer_vec.extend_from_slice(&VarNumber::from(6 as u64).to_bytes());
        outer_vec.extend_from_slice(&VarNumber::from((addr_vec.len() + data_vec.len()) as u64).to_bytes());
        outer_vec.extend_from_slice(&addr_vec);
        outer_vec.extend_from_slice(&data_vec);

        stream.write(&outer_vec).unwrap();
    }
}

fn process_mgmt(
    packet: Arc<UdpPacket>,
    chan_out: &Sender<(Vec<u8>, SocketAddr)>,
    chans_pipeline: &Vec<Sender<Arc<UdpPacket>>>,
) {
    println!("Got a management packet, {:?}", packet.data);
}