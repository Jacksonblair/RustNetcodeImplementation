use std::slice::from_raw_parts;

use crate::protocol::ProtocolError;

use super::streams::{ReadStream, Stream, WriteStream};

const PACKET_BUFFER_SIZE: u32 = 256;
const MAX_FRAGMENT_SIZE: u32 = 1024;
const MAX_FRAGMENTS_PER_PACKET: u32 = 256;
pub const MAX_PACKET_SIZE: u32 = MAX_FRAGMENT_SIZE * MAX_FRAGMENTS_PER_PACKET;

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

/** TODO */
#[derive(Copy, Clone)]
pub enum PacketTypes {
    TestPacketA,
}

/**
 * Objects have a two serialize functions for reading and writing an object
 */
pub trait Object {
    // fn serialize(&mut self, stream: &mut dyn Stream) -> bool;
    fn serialize_internal_r(&mut self, stream: &mut ReadStream) -> bool;
    fn serialize_internal_w(&mut self, stream: &mut WriteStream) -> bool;
}

pub trait Packet: Object {
    fn get_packet_type(&self) -> u32;
}

/** TODO */
pub trait PacketFactory {
    fn get_num_packet_types(&self) -> u32;
    fn get_num_allocated_packets(&self) -> u32;
    fn create_packet(&self, packet_type: u32) -> Box<dyn Packet>;
    fn destroy_packet(&self);
}

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

/** TODO */
struct PacketBufferEntry {
    sequence: u32,           // packet sequence number
    num_fragments: u32,      // number of fragments for this packet
    received_fragments: u32, // number of received fragments so far
    fragment_size: [u32; MAX_FRAGMENTS_PER_PACKET as usize], // size of fragment n in bytes
                             // uint8_t *fragmentData[MaxFragmentsPerPacket];       // pointer to data for fragment n
}

/**
    TODO
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

/** TODO */
pub fn write_packet(
    info: &PacketInfo,
    packet: &mut dyn Packet,
    buffer: &mut Vec<u32>,
    buffer_length: usize,
    header: Option<&mut dyn Object>,
) -> u32 {
    assert!(buffer.len() > 0);
    assert!(buffer.len() <= buffer_length);

    let num_packet_types = info.packet_factory.get_num_packet_types();
    let buffer_length = buffer.len();
    let mut stream = WriteStream::new(buffer, buffer.len());

    let mut crc_32: u32 = 0;
    // stream.SetContext(info.context);

    // Serialize prefix bytes
    for _i in 0..info.prefix_bytes {
        let mut zero: u32 = 0;
        stream.serialize_bits(&mut zero, 8);
    }

    // Serialize space for crc32, which we calculate at the end of writing the packet.
    if !info.raw_format {
        stream.serialize_bits(&mut crc_32, 32);
    }

    // Write header if there is one
    match header {
        Some(header) => {
            header.serialize_internal_w(&mut stream);
        }
        _ => (),
    }

    let mut packet_type = packet.get_packet_type() as i32;
    assert!(num_packet_types > 0);

    // If we have more than one packet type, serialize the packet type into the buffer?
    if num_packet_types > 1 {
        stream.serialise_int(&mut packet_type, 0, num_packet_types as i32);
    }

    // Serialize the packet
    if !packet.serialize_internal_w(&mut stream) {
        return 0;
    }

    stream.serialize_check(&mut String::from("end of packet"));
    stream.writer.flush();
    let bytes_processed = stream.get_bytes_processed();

    if ProtocolError::None != stream.get_error() {
        return 0;
    }

    // Write crc32 into packet
    if !info.raw_format {
        unsafe {
            // Pointer to buffer start + prefix bytes
            let dest_ptr = (buffer.as_mut_ptr() as *mut u8).add(info.prefix_bytes as usize);
            let protocol_bytes_temp = info.protocol_id.to_le_bytes();
            let protocol_bytes = protocol_bytes_temp.as_slice();
            let buffer_bytes =
                from_raw_parts::<u8>(dest_ptr, buffer_length - info.prefix_bytes as usize);

            let mut crc_bytes: Vec<u8> = vec![];
            crc_bytes.extend_from_slice(protocol_bytes);
            crc_bytes.extend_from_slice(buffer_bytes);

            crc_32 = crc32fast::hash(&crc_bytes);
            let src_ptr = crc_32.to_le_bytes().as_ptr();

            // Write crc32 directly into buffer
            std::ptr::copy_nonoverlapping(src_ptr, dest_ptr, 4);
        }
    }

    return bytes_processed;
}

pub fn read_packet(
    info: &PacketInfo,
    buffer: &mut Vec<u32>,
    header: Option<&mut dyn Object>,
    error: &mut ProtocolError,
) -> Option<Box<dyn Packet>> {
    assert!(buffer.len() > 0);

    if *error != ProtocolError::None {
        *error = ProtocolError::None;
    }

    let buffer_ptr = buffer.as_ptr() as *mut u8;
    let buffer_length = buffer.len();
    let mut stream = ReadStream::new(buffer, buffer.len());
    // stream.SetContext(info.context);

    for _i in 0..info.prefix_bytes {
        let mut dummy: u32 = 0;
        stream.serialize_bits(&mut dummy, 8);
    }

    // TODO: Move crc32 stuff into functions.
    let mut read_crc32 = 0;
    if !info.raw_format {
        stream.serialize_bits(&mut read_crc32, 32);

        // On read, we skip prefix bytes
        // Overwrite CRC with 0's
        // Read the rest.

        unsafe {
            let src_ptr = buffer_ptr.add(info.prefix_bytes as usize + 4);
            let protocol_bytes_temp = info.protocol_id.to_le_bytes();
            let protocol_bytes = protocol_bytes_temp.as_slice();
            let buffer_bytes =
                from_raw_parts::<u8>(src_ptr, buffer_length - info.prefix_bytes as usize - 4);
            let mut crc_bytes: Vec<u8> = vec![];
            crc_bytes.extend_from_slice(protocol_bytes);
            crc_bytes.extend_from_slice(&[0, 0, 0, 0]); // Fill in space that CRC32 was in
            crc_bytes.extend_from_slice(buffer_bytes);

            let crc_32 = crc32fast::hash(&crc_bytes);

            assert_eq!(
                read_crc32, crc_32,
                "Corrupt packet. Expected CRC32: {:?}, got CRC32: {:?}",
                read_crc32, crc_32
            );
        }
    }

    match header {
        Some(header) => {
            if !header.serialize_internal_r(&mut stream) {
                if *error != ProtocolError::None {
                    *error = ProtocolError::SerializeHeaderFailed
                }
            }
        }
        None => (),
    }

    let mut packet_type: u32 = 0;
    let num_packet_types = info.packet_factory.get_num_packet_types();
    assert!(num_packet_types > 0);

    if num_packet_types > 1 {
        let mut temp_packet_type: i32 = 0;
        if !stream.serialise_int(&mut temp_packet_type, 0, num_packet_types as i32) {
            if *error != ProtocolError::None {
                *error = ProtocolError::InvalidPacketType;
                return None;
            }
        }
        packet_type = temp_packet_type as u32;
    }

    if !info.allowed_packet_types.contains(&packet_type) {
        if *error == ProtocolError::None {
            *error = ProtocolError::PacketTypeNotAllowed;
        }
        return None;
    }

    let mut packet = info.packet_factory.create_packet(packet_type);

    if !packet.serialize_internal_r(&mut stream) {
        if *error == ProtocolError::None {
            *error = ProtocolError::SerializePacketFailed;
        }
        // info.packetFactory->DestroyPacket(packet);
    }

    if !stream.serialize_check(&mut String::from("end of packet")) {
        if *error == ProtocolError::None {
            *error = ProtocolError::SerializeCheckFailed;
        }
        // info.packetFactory->DestroyPacket(packet);
    }

    if stream.get_error() != ProtocolError::None {
        if *error == ProtocolError::None {
            *error = stream.get_error();
        }
        // info.packetFactory->DestroyPacket(packet);
    }

    return Some(packet);
}

mod tests {}
