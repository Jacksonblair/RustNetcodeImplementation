use crate::bits_required;
use super::bitpacker::{BitWriter, BitReader};


pub trait Stream {
    fn is_reading(&self) -> bool;
    fn is_writing(&self) -> bool;
    fn serialise_int(&mut self, value: &mut u32, min: u32, max: u32) -> bool;
}

pub struct WriteStream<'a> {
    writer: BitWriter<'a>
}

impl<'a> WriteStream<'a>{
    pub fn new(buffer: &mut Vec<u32>) -> WriteStream {
        return WriteStream { writer: BitWriter::new(buffer, 100) };
    }
}

impl <'a>Stream for WriteStream<'a> {
    fn is_reading(&self) -> bool {
        false
    }

    fn is_writing(&self) -> bool {
        true
    }

    fn serialise_int(&mut self, value: &mut u32, min: u32, max: u32) -> bool {
        let bits: u32 = bits_required!(min, max);
        self.writer.write_bits(*value, bits);
        return true;
    }
}

pub struct ReadStream<'a> {
    reader: BitReader<'a>
}

impl<'a> ReadStream<'a>{
    pub fn new(buffer: &mut Vec<u32>) -> ReadStream {
        return ReadStream { reader: BitReader::new(buffer, 100) };
    }
}

impl <'a>Stream for ReadStream<'a> {
    fn is_reading(&self) -> bool {
        true
    }

    fn is_writing(&self) -> bool {
        false
    }

    fn serialise_int(&mut self, value: &mut u32, min: u32, max: u32) -> bool {
        println!("Val: {:?}", value);
        println!("Min: {:?}", min);
        println!("Max: {:?}", max);

        assert!(min < max);
        let bits = bits_required!(min, max);

        if self.reader.would_overflow(bits) {
            println!("Error: There are not enough bits to read {:?} bits", bits);
            return false;
        }

        let val: u32 = self.reader.read_bits(bits);
        *value = val + min; // Add minimum. Signed values?

        self.reader.num_bits_read += bits;
        return true
    }
}