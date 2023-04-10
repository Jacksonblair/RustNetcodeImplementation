use super::streams::{ReadStream, Stream, WriteStream};

const PACKET_BUFFER_SIZE: u32 = 256;
const MAX_FRAGMENT_SIZE: u32 = 1024;
const MAX_FRAGMENTS_PER_PACKET: u32 = 256;
const MAX_PACKET_SIZE: u32 = MAX_FRAGMENT_SIZE * MAX_FRAGMENTS_PER_PACKET;

pub mod test_packets;

/*
    -- Packet Fragmentation --

    WRITING
    - Create a big ass packet
    - Write the packet
    - Check if we've written more than we can fit into a single packet
    - If true, split the packet into fragments and process them one by one
    - Else just process the single packet

    - !! READING !!
    - Get our packets in a buffer
    - COunt how many we have
    - Iterate over all of the packets
    - Read each packet, and do some safey tests.

*/

#[derive(Copy, Clone)]
pub enum PacketTypes {
    TestPacketA,
}

pub trait Object {
    fn serialize_internal_r(&mut self, stream: &mut ReadStream) -> bool;
    fn serialize_internal_w(&mut self, stream: &mut WriteStream) -> bool;
}

pub trait Packet: Object {
    fn get_packet_type(&self) -> PacketTypes;
}

pub struct PacketFactory {
    num_packet_types: u32,
    num_allocated_packets: u32,
}

impl PacketFactory {
    pub fn create_packet<T: Packet>(packet_type: PacketTypes) -> T {
        todo!();
    }

    pub fn destroy_packet() {
        todo!();
    }

    pub fn get_num_packet_types() {
        todo!()
    }
}

pub struct PacketInfo {
    raw_format: bool, // if true packets are written in "raw" format without crc32 (useful for encrypted packets).
    prefix_bytes: u32, // prefix this number of bytes when reading and writing packets. stick your own data there.
    protocol_id: u32, // protocol id that distinguishes your protocol from other packets sent over UDP.
    allowed_packet_types: Vec<PacketTypes>, // array of allowed packet types. if a packet type is not allowed the serialize read or write will fail.
                                            // packet_factory: PacketFactory,       // create packets and determine information about packet types. required.
                                            // context: Something??                 // context for the packet serialization (optional, pass in NULL)
}

struct PacketBufferEntry {
    sequence: u32,           // packet sequence number
    num_fragments: u32,      // number of fragments for this packet
    received_fragments: u32, // number of received fragments so far
    fragment_size: [u32; MAX_FRAGMENTS_PER_PACKET as usize], // size of fragment n in bytes
                             // uint8_t *fragmentData[MaxFragmentsPerPacket];       // pointer to data for fragment n
}

/*
    Packet buffer is used to:
    - Process: Send packets
    - Receive: Receive packets

*/
pub struct PacketBuffer {
    current_sequence: u16,       // sequence number of most recent packet in buffer
    num_buffered_fragments: u32, // total number of fragments stored in the packet buffer (across *all* packets)
    valid: [u8; PACKET_BUFFER_SIZE as usize], // true if there is a valid buffered packet entry at this index
    entries: [PacketBufferEntry; PACKET_BUFFER_SIZE as usize], // buffered packets in range [ current_sequence - PacketBufferSize + 1, current_sequence ] (modulo 65536)
}

impl PacketBuffer {
    /*
        Advance the current sequence for the packet buffer forward.
        This function removes old packet entries and frees their fragments.
    */
    pub fn advance(&self, sequence: u16) {
        todo!();
    }

    pub fn process_fragment(&self) -> bool {
        // calls advance
        todo!();
        true
    }

    pub fn process_packets(&self) {
        // calls process_fragment
        todo!();
    }

    pub fn receive_packets(&self) {
        todo!()
    }
}
