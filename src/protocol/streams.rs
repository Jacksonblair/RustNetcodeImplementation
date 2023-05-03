use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use super::{
    bitpacker::{BitReader, BitWriter},
    hash_string, ProtocolError,
};
use crate::bits_required;

pub trait Stream {
    fn is_reading(&self) -> bool;
    fn is_writing(&self) -> bool;
    fn serialise_int(&mut self, value: &mut i32, min: i32, max: i32) -> bool;
    fn serialize_bits(&mut self, value: &mut u32, bits: u32) -> bool;
    fn serialize_align(&mut self) -> bool;
    fn serialize_bytes(&mut self, bytes: &mut Vec<u8>, num_bytes: u32) -> bool;
    fn serialize_check(&mut self, hash: &mut String) -> bool;
    fn get_bytes_processed(&mut self) -> u32;
    fn get_error(&mut self) -> ProtocolError;
}

pub struct WriteStream<'a> {
    pub writer: BitWriter<'a>,
    error: ProtocolError,
}

impl<'a> WriteStream<'a> {
    pub fn new(buffer: &mut Vec<u32>, buffer_size: usize) -> WriteStream {
        return WriteStream {
            writer: BitWriter::new(buffer, buffer_size),
            error: ProtocolError::None,
        };
    }
}

impl<'a> Stream for WriteStream<'a> {
    fn get_error(&mut self) -> ProtocolError {
        return self.error;
    }

    fn is_reading(&self) -> bool {
        false
    }

    fn is_writing(&self) -> bool {
        true
    }

    fn serialize_bits(&mut self, value: &mut u32, bits: u32) -> bool {
        if !(bits > 0) {
            false;
        }
        if !(bits <= 32) {
            false;
        }
        self.writer.write_bits(*value, bits);
        return true;
    }

    /** serialize_int will write the value (minus the min value to save space) */
    fn serialise_int(&mut self, value: &mut i32, min: i32, max: i32) -> bool {
        assert!(min < max);
        assert!(*value >= min);
        assert!(*value <= max);

        let bits: u32 = bits_required!(min, max);
        // Convert to higher size int before subtracting to prevent overflow
        let unsigned_val = ((*value as i64) - (min as i64)) as u32;
        self.writer.write_bits(unsigned_val, bits);
        return true;
    }

    fn serialize_bytes(&mut self, bytes: &mut Vec<u8>, num_bytes: u32) -> bool {
        assert!(num_bytes > 0);
        if !self.serialize_align() {
            return false;
        }

        return self.writer.write_bytes(bytes, num_bytes);
    }

    fn serialize_align(&mut self) -> bool {
        return self.writer.write_align();
    }

    fn serialize_check(&mut self, string: &mut String) -> bool {
        self.serialize_align();
        let mut hash = hash_string(string);
        self.serialize_bits(&mut hash, 32);
        true
    }

    fn get_bytes_processed(&mut self) -> u32 {
        return self.writer.get_bytes_written();
    }
}

pub struct ReadStream<'a> {
    reader: BitReader<'a>,
    error: ProtocolError,
}

impl<'a> ReadStream<'a> {
    pub fn new(buffer: &mut Vec<u32>, buffer_size: usize) -> ReadStream {
        return ReadStream {
            reader: BitReader::new(buffer, buffer_size),
            error: ProtocolError::None,
        };
    }
}

impl<'a> Stream for ReadStream<'a> {
    fn get_error(&mut self) -> ProtocolError {
        return self.error;
    }

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
            self.error = ProtocolError::StreamOverflow;
            return false;
        }

        let unsigned_val: u32 = self.reader.read_bits(bits);

        *value = (unsigned_val as i64 + min as i64) as i32; // Add minimum back to unsigned value.
        return true;
    }

    fn serialize_bits(&mut self, value: &mut u32, bits: u32) -> bool {
        assert!(bits > 0);
        assert!(bits <= 32);
        if self.reader.would_read_past_end(bits) {
            self.error = ProtocolError::StreamOverflow;
            return false;
        }
        *value = self.reader.read_bits(bits);

        return true;
    }

    fn serialize_bytes(&mut self, bytes: &mut Vec<u8>, num_bytes: u32) -> bool {
        assert!(self.serialize_align());
        assert!(bytes.len() == num_bytes as usize);

        if self.reader.would_read_past_end(num_bytes * 8) {
            self.error = ProtocolError::StreamOverflow;
            return false;
        }

        self.reader.read_bytes(bytes, num_bytes);
        self.reader.num_bits_read += num_bytes * 8;
        true
    }

    fn serialize_align(&mut self) -> bool {
        let align_bits = self.reader.get_align_bits();
        if self.reader.would_read_past_end(align_bits) {
            self.error = ProtocolError::StreamOverflow;
            return false;
        }
        return self.reader.read_align();
    }

    fn serialize_check(&mut self, string: &mut String) -> bool {
        self.serialize_align();
        let mut val: u32 = 0;
        self.serialize_bits(&mut val, 32);
        let magic = hash_string(string);

        if magic != val {
            println!(
                "Serialize check failed: {:?}. Expected {:?}, got {:?}",
                string, magic, val
            );
            return magic == val;
        }

        true
    }

    fn get_bytes_processed(&mut self) -> u32 {
        return (self.reader.num_bits_read + 7) / 8;
    }
}
