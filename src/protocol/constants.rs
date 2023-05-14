pub const PROTOCOL_ID: u32 = 0x55667788;

pub const PACKET_BUFFER_SIZE: usize = 256;
pub const MAX_FRAGMENT_SIZE: usize = 1024;
pub const MAX_FRAGMENTS_PER_PACKET: usize = 256;
pub const MAX_PACKET_SIZE: usize = MAX_FRAGMENT_SIZE * MAX_FRAGMENTS_PER_PACKET;
pub const PACKET_FRAGMENT_HEADER_BYTES: usize = 16;
pub const MAX_PACKET_FRAGMENT_SIZE: usize = MAX_FRAGMENT_SIZE + PACKET_FRAGMENT_HEADER_BYTES;

pub type Buffer = Vec<u8>;

pub enum PacketTypes {
    FRAGMENT = 0,
    A = 1,
    B = 2,
    C = 3,
    NUM_TYPES = 4,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ProtocolError {
    None = 0,
    StreamOverflow = 1,
    SerializeHeaderFailed = 2,
    InvalidPacketType = 3,
    PacketTypeNotAllowed = 4,
    CreatePacketFailed = 5,
    SerializePacketFailed = 6,
    SerializeCheckFailed = 7,
}
