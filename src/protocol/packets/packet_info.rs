use super::packet_factory::PacketFactory;

/** TODO */
pub struct PacketInfo<'a> {
    pub raw_format: bool, // if true packets are written in "raw" format without crc32 (useful for encrypted packets).
    pub prefix_bytes: u32, // prefix this number of bytes when reading and writing packets. stick your own data there.
    pub protocol_id: u32, // protocol id that distinguishes your protocol from other packets sent over UDP.
    pub allowed_packet_types: Vec<u32>, // array of allowed packet types. if a packet type is not allowed the serialize read or write will fail.
    pub packet_factory: &'a dyn PacketFactory, // create packets and determine information about packet types. required.
                                               // context: Something??                 // context for the packet serialization (optional, pass in NULL)
}

impl PacketInfo<'_> {
    pub fn new(packet_factory: &dyn PacketFactory) -> PacketInfo {
        return PacketInfo {
            raw_format: false,
            prefix_bytes: 0,
            protocol_id: 0,
            allowed_packet_types: vec![],
            packet_factory,
        };
    }
}
