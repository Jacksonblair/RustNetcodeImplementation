use crate::{
    bits_required,
    protocol::{
        bitpacker::bit_writer::BitWriter,
        constants::{Buffer, ProtocolError},
        helpers::hash_string,
    },
};

use super::Stream;

pub struct WriteStream<'a> {
    pub writer: BitWriter<'a>,
    error: ProtocolError,
}

impl<'a> WriteStream<'a> {
    pub fn new(buffer: &mut Buffer, buffer_size: usize) -> WriteStream {
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

    /** Serializes as many 0's as it needs to align buffer data to next byte */
    fn serialize_align(&mut self) -> bool {
        return self.writer.write_align();
    }

    /** Pads buffer data to next byte and serializes string hashed to 32 bits */
    fn serialize_check(&mut self, string: &mut String) -> bool {
        // println!("------------ START");
        // self.writer.print_word(self.writer.get_word_index());

        self.serialize_align();
        let mut hash = hash_string(string);
        self.serialize_bits(&mut hash, 32);

        // self.writer.print_word(self.writer.get_word_index() - 1);
        // self.writer.print_word(self.writer.get_word_index());
        // println!("------------ END");
        true
    }

    fn get_bytes_processed(&mut self) -> u32 {
        return self.writer.get_bytes_written();
    }
}
