use crate::bits_required;
use super::bitpacker::{BitWriter, BitReader};

pub trait Stream {
    fn is_reading(&self) -> bool;
    fn is_writing(&self) -> bool;
    fn serialise_int(&mut self, value: &mut i32, min: i32, max: i32) -> bool;
    fn serialize_bits(&mut self, value: &mut u32, bits: u32) -> bool;
    fn serialize_align(&mut self) -> bool;
    fn serialize_bytes(&mut self, bytes: &mut [u8], num_bytes: u32) -> bool;
}

pub struct WriteStream<'a> {
    pub writer: BitWriter<'a>
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

    fn serialize_bits(&mut self, value: &mut u32, bits: u32) -> bool {
        if !(bits > 0) { false; }
        if !(bits <= 32) { false; }
        self.writer.write_bits(*value, bits);
        return true;
    }

    fn serialise_int(&mut self, value: &mut i32, min: i32, max: i32) -> bool {
        /*
            assert( min < max );
            assert( value >= min );
            assert( value <= max );
        */
        let bits: u32 = bits_required!(min, max);
        let unsigned_val = (*value - min) as u32;
        self.writer.write_bits(unsigned_val, bits);
        return true;
    }

    fn serialize_bytes(&mut self, bytes: &mut [u8], num_bytes: u32) -> bool {

        assert!(num_bytes > 0);
        if !self.serialize_align() {
            return false;
        }

        return self.writer.write_bytes(bytes, num_bytes);
    }

    fn serialize_align(&mut self) -> bool {
        return self.writer.write_align();
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

    fn serialise_int(&mut self, value: &mut i32, min: i32, max: i32) -> bool {
        assert!(min < max);
        let bits = bits_required!(min, max);

        if self.reader.would_read_past_end(bits) {
            return false;
        }

        let unsigned_val: u32 = self.reader.read_bits(bits);
        *value = unsigned_val as i32 + min; // Add minimum back to unsigned value.
        return true
    }

    fn serialize_bits(&mut self, value: &mut u32, bits: u32) -> bool {
        if !(bits > 0) { false; }
        if !(bits <= 32) { false; }
        if self.reader.would_read_past_end(bits) {
            false;
        }
        *value = self.reader.read_bits(bits);
        return true
    }

    fn serialize_bytes(&mut self, bytes: &mut [u8], num_bytes: u32) -> bool {

        assert!(self.serialize_align());
        assert!(bytes.len() == num_bytes as usize);

        if self.reader.would_read_past_end(num_bytes * 8) {
            // Throw some overflow error?
            return false;
        }

        self.reader.read_bytes(bytes, num_bytes);
        self.reader.num_bits_read += num_bytes * 8;
        true
    }

    fn serialize_align(&mut self) -> bool {
        return self.reader.read_align();
    }
}

