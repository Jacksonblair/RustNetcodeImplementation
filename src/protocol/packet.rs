use super::{streams::Stream};

trait Serialise {
    fn serialise(&mut self, stream: &mut impl Stream) -> bool;
}

pub struct Packet {
    packet_type: u32
}

impl Packet {
    fn new(packet_type: u32) -> Packet {
        return Packet { packet_type }
    }
}

impl Serialise for Packet {
    fn serialise(&mut self, stream: &mut impl Stream) -> bool {
        return true
    }
}

pub struct PacketFactory {
    num_packet_types: u32
}

impl PacketFactory {

    pub fn new(num_packet_types: u32) -> PacketFactory {
        return PacketFactory { num_packet_types }
    }

    pub fn create_packet(&self, packet_type: u32) -> Packet {
        Packet::new(packet_type)
    }
}