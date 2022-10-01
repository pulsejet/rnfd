mod fib;

use std::io::Read;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::{sync::Arc, net::SocketAddr};
use crossbeam::channel::{Receiver, Sender};
use crate::socket::UdpPacket;
use crate::table::Table;
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
        read_yanfd(stream_arc_read, chan_out, chans_pipeline)
    });
}

fn read_yanfd(
    stream_arc: Arc<UnixStream>,
    chan_out: Sender::<(Vec<u8>, SocketAddr)>,
    chans_pipeline: Vec<Sender<Arc<UdpPacket>>>,
) {
    loop {
        let mut buf = [0; 8800];
        let mut stream = &*stream_arc;
        let len = stream.read(&mut buf).unwrap();
        if len == 0 {
            panic!("YaNFD socket closed");
        }

        let frame = &buf[..len];
        read_yanfd_frame(frame, &chan_out, &chans_pipeline);
    }
}

fn read_yanfd_frame(
    frame: &[u8],
    chan_out: &Sender::<(Vec<u8>, SocketAddr)>,
    chans_pipeline: &Vec<Sender<Arc<UdpPacket>>>,
) {
    let frame_tlo = tlv::vec_decode::read_tlo(frame);
    if frame_tlo.is_err() {
        return;
    }
    let frame_tlo = frame_tlo.unwrap();

    let c_frame = &frame[frame_tlo.o..];
    if frame_tlo.t == 6 {
        let res = read_yanfd_data_frame(c_frame);
        match res {
            Ok((addr, data)) => {
                println!("YaNFD: read {} bytes for {:?}", data.len(), addr);
                chan_out.send((data, addr)).unwrap();
            },
            Err(e) => {
                println!("YaNFD: parsing error {:?}", e);
            }
        }
    } else if frame_tlo.t == 3 {
        read_yanfd_mgmt_frame(c_frame, &chans_pipeline);
    }
}

fn read_yanfd_mgmt_frame(frame: &[u8], chans_pipeline: &Vec<Sender<Arc<UdpPacket>>>) {
    let pack = Arc::new(UdpPacket {
        data: frame.to_vec(),
        addr: SocketAddr::from(([0, 0, 0, 0], 0)),
    });
    for chan in chans_pipeline {
        chan.send(pack.clone()).unwrap();
    }
}

fn read_yanfd_data_frame(frame: &[u8]) -> Result<(SocketAddr, Vec<u8>), std::io::Error> {
    // Get address (TLV type 4)
    let addr_tlo = tlv::vec_decode::read_tlo(frame)?;
    let addr = read_addr(&frame)?;

    // Peel off link layer header
    let mut data = &frame[addr_tlo.o+addr_tlo.l as usize..];
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

pub fn read_addr(frame: &[u8]) -> Result<SocketAddr, std::io::Error> {
    let addr_tlo = tlv::vec_decode::read_tlo(frame)?;
    if addr_tlo.t != 4 {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Expected TLV type 4"));
    }
    let addr = &frame[addr_tlo.o..addr_tlo.o+addr_tlo.l as usize];
    let addr_str = std::str::from_utf8(addr);
    if addr_str.is_err() {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Invalid address"));
    }
    let addr = addr_str.unwrap().parse::<SocketAddr>();
    if addr.is_err() {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Invalid address"));
    }
    let addr = addr.unwrap();
    Ok(addr)
}

pub fn process_frame(table: &mut Table, packet: Arc<UdpPacket>) {
    let tlo = tlv::vec_decode::read_tlo(&packet.data);
    if tlo.is_err() {
        return;
    }
    let tlo = tlo.unwrap();
    let frame = &packet.data[tlo.o..];

    let res;
    if tlo.t == 1 {
        res = fib::read_insert_hop(table, frame);
    } else {
        res = Ok(());
        println!("Unknown MGMT frame type {}", tlo.t);
    }

    if res.is_err() {
        println!("YaNFD: Error processing MGMT frame type {}: {:?}", tlo.t, res);
    }
}