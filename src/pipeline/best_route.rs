use std::{sync::Arc, borrow::Borrow};
use crate::{table::Table, socket::UdpPacket};
use super::{strategy::Strategy, Interest};

pub struct BestRouteStrategy {}

impl Strategy for BestRouteStrategy {
    fn after_receive_interest(table: &mut Table, packet: Arc<UdpPacket>, interest: Interest) {
        let mut res_hops = Vec::new();

        let nexthops = interest.nexthops.as_ref().unwrap();
        if nexthops.len() > 0 {
            let mut nexthop = &nexthops[0];
            let mut cost = nexthop.cost;
            for n in nexthops {
                if n.cost < cost {
                    nexthop = n;
                    cost = n.cost;
                }
            }
            res_hops.push(nexthop.clone());
        } else {
            // TODO: send NACK
            println!("No nexthops");
            return;
        }

        // Call back to forwarder pipeline
        super::interest::on_outgoing_interest(table, packet, interest, res_hops);
    }
}