use std::sync::Arc;
use crate::{table::Table, socket::UdpPacket};
use super::Interest;

pub trait Strategy {
    fn after_receive_interest(table: &mut Table, packet: Arc<UdpPacket>, interest: Interest);
}