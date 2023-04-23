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

/** TODO */
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
    // header: &mut dyn Object,
) -> u32 {
    assert!(buffer.len() > 0);

    let num_packet_types = info.packet_factory.get_num_packet_types();
    let mut stream = WriteStream::new(buffer, buffer.len());

    // stream.SetContext(info.context);

    // Serialize prefix bytes
    // for _i in 0..info.prefix_bytes {
    //     let mut zero: u32 = 0;
    //     stream.serialize_bits(&mut zero, 8);
    // }

    // Serialize crc32
    // let mut crc32: u32 = 0;
    // if !info.raw_format {
    //     stream.serialize_bits(&mut crc32, 32);
    // }

    // if (header)
    // {
    //     if (!header->SerializeInternal(stream))
    //         return 0;
    // }

    let mut packet_type = packet.get_packet_type() as i32;
    assert!(num_packet_types > 0);

    // If we have more than one packet type, serialize the packet type into the buffer?
    if num_packet_types > 1 {
        stream.serialise_int(&mut packet_type, 0, num_packet_types as i32);
        println!("WROTE PACKET TYPE: {:?}", packet_type);
    }

    println!("{:?}", stream.writer.get_bits_written());

    // Serialize the packet
    if !packet.serialize_internal_w(&mut stream) {
        return 0;
    }

    // stream.SerializeCheck("end of packet");

    stream.writer.flush();

    if !info.raw_format {
        // uint32_t network_protocolId = host_to_network(info.protocolId);
        // crc32 = calculate_crc32((uint8_t *)&network_protocolId, 4);
        // crc32 = calculate_crc32(buffer + info.prefixBytes, stream.GetBytesProcessed() - info.prefixBytes, crc32);
        // *((uint32_t *)(buffer + info.prefixBytes)) = host_to_network(crc32);
    }

    // if (stream.GetError())
    //     return 0;

    // return stream
    return stream.get_bytes_processed();
}

/** TODO */
pub fn read_packet(
    info: &PacketInfo,
    buffer: &mut Vec<u32>,
    // header: &mut dyn Object,
) -> Box<dyn Packet> {
    assert!(buffer.len() > 0);

    for i in 0..3 {
        println!("READ buffer: {:#034b}", buffer[i]);
    }

    //if (errorCode)
    //*errorCode = PROTOCOL2_ERROR_NONE;

    let mut stream = ReadStream::new(buffer, buffer.len());
    // stream.SetContext(info.context);

    // for i in 0..info.prefix_bytes {
    //     let dummy: u32 = 0;
    //     stream.serialize_bits(&mut dummy, 8);
    // }

    let read_crc32 = 0;
    if info.raw_format {
        // uint32_t network_protocolId = host_to_network(info.protocolId);
        // uint32_t crc32 = calculate_crc32((const uint8_t *)&network_protocolId, 4);
        // uint32_t zero = 0;
        // crc32 = calculate_crc32((const uint8_t *)&zero, 4, crc32);
        // crc32 = calculate_crc32(buffer + info.prefixBytes + 4, bufferSize - 4 - info.prefixBytes, crc32);

        // if (crc32 != read_crc32)
        // {
        //     printf("corrupt packet. expected crc32 %x, got %x\n", crc32, read_crc32);

        //     if (errorCode)
        //         *errorCode = PROTOCOL2_ERROR_CRC32_MISMATCH;
        //     return NULL;
        // }
    }

    // if (header)
    // {
    //     if (!header->SerializeInternal(stream))
    //     {
    //         if (errorCode)
    //             *errorCode = PROTOCOL2_ERROR_SERIALIZE_HEADER_FAILED;
    //         return NULL;
    //     }
    // }

    let mut packet_type: u32 = 0;
    let num_packet_types = info.packet_factory.get_num_packet_types();

    assert!(num_packet_types > 0);

    if num_packet_types > 1 {
        let mut temp_packet_type: i32 = 0;
        if !stream.serialise_int(&mut temp_packet_type, 0, num_packet_types as i32) {}
        packet_type = temp_packet_type as u32;
        // if (!stream.SerializeInteger(packetType, 0, numPacketTypes - 1))
        // {
        //     if (errorCode)
        //         *errorCode = PROTOCOL2_ERROR_IN&mut packet_type;
        //     return NULL;
        // }
        println!("READ PACKET TYPE: {:?}", packet_type);
    }

    if info.allowed_packet_types.contains(&packet_type) {
        // if (errorCode)
        //  *errorCode = PROTOCOL2_ERROR_PACKET_TYPE_NOT_ALLOWED;
    }

    let mut packet = info.packet_factory.create_packet(packet_type);

    if !packet.serialize_internal_r(&mut stream) {
        println!("Failed to serialize read")
    }

    // if !packet.serialize_internal_r(&mut stream) {
    //  if (errorCode)
    //      *errorCode = PROTOCOL2_ERROR_SERIALIZE_PACKET_FAILED;
    //      goto cleanup;
    // }
    // cleanup:
    //     info.packetFactory->DestroyPacket(packet);
    //     return NULL;
    // }

    packet
}

mod tests {}
