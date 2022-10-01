use std::{collections::{HashMap, LinkedList}, net::SocketAddr, rc::Rc, cell::RefCell };
use crate::{pipeline::Interest, tlv::vec_decode};

#[derive(Debug, Clone, Copy)]
pub struct NextHop {
    pub addr: SocketAddr,
    pub cost: u64,
}

pub struct PITNode {
    pub name: Vec<u8>,
    pub children: HashMap<u64, Rc<RefCell<PITNode>>>,
    pub in_records: LinkedList<InRecord>,
    pub out_records: HashMap<u64, OutRecord>,
    pub strategy: u64,
    pub nexthops: Vec<NextHop>,
}

impl PITNode {
    pub fn new(name: Vec<u8>) -> PITNode {
        PITNode {
            name: name,
            children: HashMap::new(),
            in_records: LinkedList::new(),
            out_records: HashMap::new(),
            strategy: 0,
            nexthops: Vec::new(),
        }
    }

    pub fn insert_hop(&mut self, hop: NextHop) {
        // Look for existing hop
        for i in 0..self.nexthops.len() {
            if self.nexthops[i].addr == hop.addr {
                self.nexthops[i].cost = hop.cost;
                return;
            }
        }
        self.nexthops.push(hop);
    }
}

#[derive(Debug)]
pub struct InRecord {
    pub expiry: u64,
    pub face: std::net::SocketAddr,
    pub can_be_prefix: Option<bool>,
    pub must_be_fresh: Option<bool>,
    pub nonce: Option<u32>,
    pub lifetime: Option<u64>,
    pub hop_limit: Option<u8>,
}

impl InRecord {
    pub fn new(interest: &Interest, face: SocketAddr) -> InRecord {
        InRecord {
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

#[derive(Debug)]
pub struct OutRecord {
    pub face: std::net::SocketAddr,
    pub nonce: u32,
    pub timestamp: u64,
    // nacked_field: Vec<u64>,
}

pub struct PIT {
    root: Rc<RefCell<PITNode>>,
}

impl PIT {
    pub fn new() -> PIT {
        PIT {
            root: Rc::new(RefCell::new(PITNode::new(Vec::new()))),
        }
    }

    /**
     * Add a name node to the PIT or get matching node
     * Returns (node, strategy, nexthops)
     */
    pub fn insert_or_get(&mut self, name: &Vec<u8>) -> Result<
        (Rc<RefCell<PITNode>>, u64, Vec<NextHop>),
        std::io::Error>
    {
        let mut o = 0; // offset in name
        let mut node_ref = self.root.clone();
        let mut strategy = 0;
        let mut nexthops = Vec::new();

        while o < name.len() {
            let tlo = vec_decode::read_tlo(&name[o..])?;

            node_ref = {
                let mut node = node_ref.borrow_mut();

                if node.strategy > 0 {
                    strategy = node.strategy;
                }
                if node.nexthops.len() > 0 {
                    nexthops = node.nexthops.clone();
                }

                if o + tlo.o + tlo.l as usize > name.len() {
                    return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Incorrect name TLV encoding"));
                }
                let n_name = &name[o..o+tlo.o+tlo.l as usize];

                let n_hash = fasthash::metro::hash64(n_name);
                let enode = node.children.get(&n_hash);

                match enode {
                    Some(n) => { n.clone() },
                    None => {
                        let n = Rc::new(RefCell::new(PITNode::new(n_name.to_vec())));
                        node.children.insert(n_hash, n.clone());
                        n
                    }
                }
            };

            o += tlo.o + tlo.l as usize;
        }

        return Ok((node_ref.clone(), strategy, nexthops));
    }

    /**
     * Find a name node in the PIT
     * Returns (node, strategy, nexthops)
     */
    pub fn get(&mut self, name: &Vec<u8>) -> Option<(Rc<RefCell<PITNode>>, u64, Vec<NextHop>)> {
        let mut o = 0; // offset in name
        let mut node_ref_opt = Some(self.root.clone());
        let mut strategy = 0;
        let mut nexthops = Vec::new();

        while o < name.len() {
            let tlo = vec_decode::read_tlo(&name[o..]);

            node_ref_opt = {
                let node_ref = match node_ref_opt {
                    Some(n) => n,
                    None => { return None; }
                };
                let node = node_ref.borrow();

                if node.strategy > 0 {
                    strategy = node.strategy;
                }
                if node.nexthops.len() > 0 {
                    nexthops = node.nexthops.clone();
                }

                match tlo {
                    Ok(tlo) => {
                        let n_name = &name[o..o+tlo.o+tlo.l as usize];
                        let n_hash = fasthash::metro::hash64(n_name);
                        let n = node.children.get(&n_hash);
                        o += tlo.o + tlo.l as usize;
                        match n {
                            Some(n) => { Some(n.clone()) },
                            None => { None }
                        }
                    }
                    Err(_) => { None }
                }
            };
        }

        match node_ref_opt {
            Some(n) => { Some((n, strategy, nexthops)) },
            None => { None }
        }
    }

    /**
     * Find name nodes in the PIT that match a name including CanBePrefix higher components
     * Returns nodes
     */
    pub fn get_all_can_be_pfx(&mut self, name: &Vec<u8>) -> Vec<Rc<RefCell<PITNode>>> {
        let mut o = 0; // offset in name
        let mut node_ref_opt = Some(self.root.clone());
        let mut nodes = Vec::new();

        while o < name.len() {
            let tlo = vec_decode::read_tlo(&name[o..]);

            node_ref_opt = {
                let node_ref = match node_ref_opt {
                    Some(n) => n,
                    None => { return nodes; }
                };
                let node = node_ref.borrow();

                match tlo {
                    Ok(tlo) => {
                        let n_name = &name[o..o+tlo.o+tlo.l as usize];
                        let n_hash = fasthash::metro::hash64(n_name);
                        let n = node.children.get(&n_hash);
                        o += tlo.o + tlo.l as usize;
                        match n {
                            Some(n) => {
                                nodes.push(n.clone());
                                Some(n.clone())
                            },
                            None => { None }
                        }
                    }
                    Err(_) => { None }
                }
            };
        }

        nodes
    }
}