use crate::MAX_PACKET_SIZE;

use self::packet::Packet;

pub mod packet;
pub mod streams;
pub mod macros;
pub mod bitpacker;

pub fn write_packet(packet: Packet, buffer: Vec<u8>, max_packet_size: usize) {

    // Create a writestream

    // add prefix bytes

    // add crc32

    // serialize packet type

    // serialize internal (this is where the data is written)
        /*
            Packet_n {
                some_game_variable

                serialize_internal(stream) {
                    write_variable(stream, variable)
                }
            }
        */


    // flush stream

}