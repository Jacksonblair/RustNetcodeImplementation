pub mod read_stream;
pub mod write_stream;
use super::constants::ProtocolError;

pub trait Stream {
    fn is_reading(&self) -> bool;
    fn is_writing(&self) -> bool;
    fn serialise_int(&mut self, value: &mut i32, min: i32, max: i32) -> bool;
    fn serialize_bits(&mut self, value: &mut u32, bits: u32) -> bool;
    fn serialize_align(&mut self) -> bool;
    fn serialize_bytes(&mut self, bytes: &mut Vec<u8>, num_bytes: u32) -> bool;
    fn serialize_check(&mut self, hash: &mut String) -> bool;
    fn get_bytes_processed(&self) -> u32;
    fn get_bits_processed(&self) -> u32;
    fn get_bits_remaining(&self) -> u32;
    fn get_error(&mut self) -> ProtocolError;
}
