use crate::protocol::streams::{read_stream::ReadStream, write_stream::WriteStream};

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
