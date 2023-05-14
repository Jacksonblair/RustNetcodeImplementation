use crate::protocol::constants::MAX_PACKET_FRAGMENT_SIZE;

#[derive(Debug)]
pub struct PacketData {
    pub size: u32,
    pub data: Vec<u8>,
}

impl PacketData {
    pub fn new() -> PacketData {
        PacketData {
            size: 0,
            data: vec![0; MAX_PACKET_FRAGMENT_SIZE as usize],
        }
    }
}
