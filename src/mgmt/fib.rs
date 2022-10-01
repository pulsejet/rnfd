use std::io;
use crate::{table::{Table, pit::NextHop}, tlv};

pub fn read_insert_hop(table: &mut Table, mut frame: &[u8]) -> Result<(), io::Error> {
    // Name
    let name_tlo = tlv::vec_decode::read_tlo(&frame[..])?;
    let name = &frame[name_tlo.o..name_tlo.o+name_tlo.l as usize].to_vec();
    frame = &frame[name_tlo.o+name_tlo.l as usize..];

    // Address
    let addr_tlo = tlv::vec_decode::read_tlo(&frame[..])?;
    let addr = super::read_addr(frame)?;
    frame = &frame[addr_tlo.o+addr_tlo.l as usize..];

    // Cost
    let cost_tlo = tlv::vec_decode::read_tlo(&frame[..])?;
    let cost = tlv::vec_decode::read_nni(&frame[cost_tlo.o..], cost_tlo.l)?;

    println!("YaNFD: Inserting hop {:?} {} {}", name, addr.to_string(), cost);

    let (node, _, _) = table.pit.insert_or_get(name)?;
    node.borrow_mut().insert_hop(NextHop { addr, cost });

    Ok(())
}