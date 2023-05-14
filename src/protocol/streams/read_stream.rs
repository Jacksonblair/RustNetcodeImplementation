use crate::{
    bits_required,
    protocol::{
        bitpacker::bit_reader::BitReader,
        constants::{Buffer, ProtocolError},
        helpers::hash_string,
    },
};

use super::Stream;

pub struct ReadStream<'a> {
    pub reader: BitReader<'a>,
    error: ProtocolError,
}

impl<'a> ReadStream<'a> {
    pub fn new(buffer: &mut Buffer, buffer_size: usize) -> ReadStream {
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
        // self.reader.print_word(self.reader.get_word_index());

        self.serialize_align();
        // self.reader.print_word(self.reader.get_word_index());
        let mut val: u32 = 0;

        self.serialize_bits(&mut val, 32);

        let hash = hash_string(string);

        if hash != val {
            println!(
                "Serialize check failed: {:?}. Expected {:?}, got {:?}",
                string, hash, val
            );
            return hash == val;
        }

        true
    }

    fn get_bytes_processed(&self) -> u32 {
        return (self.reader.num_bits_read + 7) / 8;
    }

    fn get_bits_processed(&self) -> u32 {
        return self.reader.get_bits_read();
    }

    fn get_bits_remaining(&self) -> u32 {
        return self.reader.get_bits_remaining();
    }
}
