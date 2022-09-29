use std::{collections::{HashMap, LinkedList}, net::SocketAddr };
use crate::{pipeline::Interest, tlv::vec_decode};

#[derive(Debug, Clone)]
pub struct NextHop {
    pub addr: SocketAddr,
    pub cost: u64,
}

pub struct PITNode {
    pub name: Vec<u8>,
    pub children: HashMap<u64, PITNode>,
    pub in_records: LinkedList<PITEntry>,
    pub strategy: u64,
    pub nexthops: Vec<NextHop>,
}

impl PITNode {
    pub fn new(name: Vec<u8>) -> PITNode {
        PITNode {
            name: name,
            children: HashMap::new(),
            in_records: LinkedList::new(),
            strategy: 0,
            nexthops: Vec::new(),
        }
    }
}

pub struct PITEntry {
    expiry: u64,
    face: std::net::SocketAddr,
    can_be_prefix: Option<bool>,
    must_be_fresh: Option<bool>,
    nonce: Option<u32>,
    lifetime: Option<u64>,
    hop_limit: Option<u8>,
}

impl PITEntry {
    pub fn new(interest: &Interest, face: SocketAddr) -> PITEntry {
        PITEntry {
            expiry: 0,
            face,
            can_be_prefix: interest.can_be_prefix,
            must_be_fresh: interest.must_be_fresh,
            nonce: interest.nonce,
            lifetime: interest.lifetime,
            hop_limit: interest.hop_limit,
        }
    }
}

pub struct PIT {
    root: PITNode,
}

impl PIT {
    pub fn new() -> PIT {
        PIT {
            root: PITNode::new(Vec::new()),
        }
    }

    /**
     * Add a name node to the PIT or get matching node
     * Returns (node, strategy, nexthops)
     */
    pub fn insert_or_get(&mut self, name: &Vec<u8>) -> Result<
        (&mut PITNode, u64, Vec<NextHop>),
        std::io::Error>
    {
        let mut o = 0; // offset in name
        let mut node = &mut self.root;
        let mut strategy = 0;
        let mut nexthops = Vec::new();

        while o < name.len() {
            if node.strategy > 0 {
                strategy = node.strategy;
            }
            if node.nexthops.len() > 0 {
                nexthops = node.nexthops.clone();
            }

            let tlo = vec_decode::read_tlo(&name[o..])?;
            let n_name = &name[o+tlo.o..o+tlo.o+tlo.l as usize];
            let n_hash = fasthash::metro::hash64(n_name);
            node = node.children.entry(n_hash).or_insert(PITNode::new(n_name.to_vec()));
            o += tlo.o + tlo.l as usize;
        }

        return Ok((node, strategy, nexthops));
    }

    /**
     * Find a name node in the PIT
     * Returns (node, strategy, nexthops)
     */
    pub fn get(&mut self, interest: &Interest) -> Option<(&mut PITNode, u64, Vec<NextHop>)> {
        let mut o = 0; // offset in name
        let mut node = &mut self.root;
        let mut strategy = 0;
        let mut nexthops = Vec::new();

        while o < interest.name.len() {
            if node.strategy > 0 {
                strategy = node.strategy;
            }
            if node.nexthops.len() > 0 {
                nexthops = node.nexthops.clone();
            }

            let tlo = vec_decode::read_tlo(&interest.name[o..]);
            match tlo {
                Ok(tlo) => {
                    let n_name = &interest.name[o+tlo.o..o+tlo.o+tlo.l as usize];
                    let n_hash = fasthash::metro::hash64(n_name);
                    let found = node.children.get_mut(&n_hash);
                    if found.is_none() {
                        return None;
                    }
                    node = found.unwrap();
                    o += tlo.o + tlo.l as usize;
                }
                Err(_) => {
                    return None;
                }
            }
        }

        return Some((node, strategy, nexthops));
    }
}