use std::{collections::{HashMap, LinkedList}, rc::Rc, net::SocketAddr };

use crate::{pipeline::{Interest, interest}, tlv::vec_decode};

pub struct PIT {
    root: PITNode,
}

pub struct PITNode {
    pub name: Vec<u8>,
    pub children: HashMap<u64, PITNode>,
    pub pending_interests: LinkedList<PITEntry>,
    pub strategy: u64,
}

impl PITNode {
    pub fn new(name: Vec<u8>) -> PITNode {
        PITNode {
            name: name,
            children: HashMap::new(),
            pending_interests: LinkedList::new(),
            strategy: 0,
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

impl PIT {
    pub fn new() -> PIT {
        PIT {
            root: PITNode::new(Vec::new()),
        }
    }

    pub fn insert_or_get(&mut self, name: &Vec<u8>) -> Result<&mut PITNode, std::io::Error> {
        let mut o = 0; // offset in name
        let mut node = &mut self.root;

        while o < name.len() {
            let tlo = vec_decode::read_tlo(&name[o..])?;
            let n_name = &name[o+tlo.o..o+tlo.o+tlo.l as usize];
            let n_hash = fasthash::metro::hash64(n_name);
            node = node.children.entry(n_hash).or_insert(PITNode::new(n_name.to_vec()));
            o += tlo.o + tlo.l as usize;
        }

        return Ok(node);
    }

    pub fn get(&self, interest: &Interest) -> Option<&PITNode> {
        let mut o = 0; // offset in name
        let mut node = &self.root;

        while o < interest.name.len() {
            let tlo = vec_decode::read_tlo(&interest.name[o..]);
            match tlo {
                Ok(tlo) => {
                    let n_name = &interest.name[o+tlo.o..o+tlo.o+tlo.l as usize];
                    let n_hash = fasthash::metro::hash64(n_name);
                    let found = node.children.get(&n_hash);
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

        return Some(&node);
    }
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