use crate::{
    impl_object_for_packet,
    protocol::{
        constants::*,
        serialization::*,
        streams::{read_stream::ReadStream, write_stream::WriteStream, Stream},
    },
};

use super::object::Object;
use super::object::Packet;

// fragment packet on-the-wire format:
// [crc32] (32 bits) | [sequence] (16 bits) | [packet type 0] (# of bits depends on number of packet types)
// [fragment id] (8 bits) | [num fragments] (8 bits) | (pad zero bits to nearest byte) | <fragment data>
pub struct FragmentPacket {
    // input/output
    pub fragment_size: u32, // set as input on serialize write. output on serialize read (inferred from size of packet)

    // serialized data
    pub crc32: u32,
    pub sequence: u16,
    pub packet_type: u32,
    pub fragment_id: u8,
    pub num_fragments: u8,
    pub fragment_data: Vec<u8>,
}

impl FragmentPacket {
    pub fn new() -> FragmentPacket {
        return FragmentPacket {
            fragment_size: 0,
            crc32: 0,
            sequence: 0,
            packet_type: 0,
            fragment_id: 0,
            num_fragments: 0,
            fragment_data: vec![0; MAX_PACKET_FRAGMENT_SIZE],
        };
    }

    pub fn serialize(&mut self, stream: &mut dyn Stream) -> bool {
        serialize_bits_macro(stream, &mut self.crc32, 32);
        serialize_bits_macro(stream, &mut (self.sequence as u32), 16);
        self.packet_type = 0;
        serialize_int_macro(
            stream,
            &mut (self.packet_type as i32),
            0,
            PacketTypes::NUM_TYPES as i32 - 1,
        );

        // If packet type is not fragment, then return
        if self.packet_type != 0 {
            return true;
        }

        serialize_bits_macro(stream, &mut (self.fragment_id as u32), 8);
        serialize_bits_macro(stream, &mut (self.num_fragments as u32), 8);
        serialize_align_macro(stream);

        if stream.is_reading() {
            assert!(stream.get_bits_remaining() % 8 == 0);
            self.fragment_size = stream.get_bits_remaining() / 8;
            if self.fragment_size <= 0 || self.fragment_size > MAX_FRAGMENT_SIZE as u32 {
                println!(
                    "Packet fragment size is out of bounds ({:?})",
                    self.fragment_size
                );
                return false;
            }
        }

        assert!(self.fragment_size > 0);
        assert!(self.fragment_size <= MAX_FRAGMENT_SIZE as u32);

        serialize_bytes_internal(stream, &mut self.fragment_data, self.fragment_size);
        true
    }
}

impl Packet for FragmentPacket {
    fn get_packet_type(&self) -> u32 {
        PacketTypes::FRAGMENT as u32
    }
}

impl_object_for_packet!(FragmentPacket);
