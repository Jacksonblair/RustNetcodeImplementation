pub struct BitWriter<'a> {
    buffer: &'a mut Vec<u32>,
    scratch: u64,
    num_bits: u32,
    num_words: u32,
    bits_written: u32,
    word_index: u32,
    scratch_bits: i32,
}

impl BitWriter<'_> {
    pub fn new(buffer: &mut Vec<u32>, buffer_size: usize) -> BitWriter {
        assert!(buffer_size % 4 == 0, "bytes must be a multiple of 4");
        return BitWriter {
            buffer,
            scratch: 0,
            scratch_bits: 0,
            word_index: 0,
            bits_written: 0,
            num_words: (buffer_size as u32) / 4,
            num_bits: (buffer_size as u32 / 4) * 32,
        };
    }

    // Align buffer to next byte
    pub fn write_align(&mut self) -> bool {
        let remainder_bits = self.bits_written % 8;
        if remainder_bits != 0 {
            self.write_bits(0, 8 - remainder_bits);
            return (self.bits_written % 8) == 0;
        }
        return true;
    }

    // Flush remaining bits in scratch into buffer
    pub fn flush(&mut self) {
        if self.scratch_bits != 0 {
            assert!(self.word_index < self.num_words);
            self.buffer[self.word_index as usize] = self.scratch as u32;
            self.scratch >>= 32;
            self.scratch_bits -= 32;
            self.word_index += 1;
        };
    }

    pub fn write_bits(&mut self, val_to_write: u32, num_bits: u32) {
        assert!(num_bits > 0);
        assert!(num_bits <= 32);
        assert!(
            self.bits_written + num_bits <= self.num_bits,
            "Not enough space remaining in buffer."
        );

        // Mask off value to specified precision
        let mask: u32 = (u64::pow(2, num_bits) - 1) as u32;
        let value = val_to_write & mask;

        // Add value to scratch, shifted left by amount of bits in scratch
        self.scratch |= ((value as u64) << self.scratch_bits) as u64;

        // Increase scratch_bits by number of bits written
        self.scratch_bits += num_bits as i32;

        /*
            If we've written 32 or more bits into the scratch
            - copy first 32 bits from scratch into buffer
            - shift scratch left by 32
            - subtract 32 from scratch_bits
            - increment word index
        */
        if self.scratch_bits >= 32 {
            assert!(self.word_index < self.num_words);

            // println!("Added: {val_to_write:#num_bits$b}", val_to_write=val_to_write, num_bits=(num_bits as usize) + 2);
            // println!("Scratch full: {:#066b}", self.scratch);
            // println!("Scratch bits: {:?}", self.scratch_bits);
            // println!("Adding to buffer: {:#034b}", self.scratch);

            self.buffer[self.word_index as usize] = self.scratch as u32;
            self.scratch >>= 32;
            self.scratch_bits -= 32;
            self.word_index += 1;

            // println!("Updated buffer: {:?}", self.buffer);
            // println!("Remaining bits: {:?}", self.scratch_bits);
        }

        self.bits_written += num_bits;
    }

    pub fn write_bytes(&mut self, bytes: &mut [u8], num_bytes: u32) -> bool {
        assert!(self.get_align_bits() == 0);
        assert!(self.bits_written + num_bytes * 8 <= self.num_bits);
        assert!(
            (self.bits_written % 32) == 0
                || (self.bits_written % 32) == 8
                || (self.bits_written % 32) == 16
                || (self.bits_written % 32) == 24
        );

        // -- Writing leading word --

        // Number of bytes to fill word
        let mut head_bytes: u32 = (4 - (self.bits_written % 32) / 8) % 4;

        // println!("HEAD BYTES {:?}", head_bytes);
        // println!("{:#034b}", self.scratch);
        // println!(".-----.");

        if head_bytes > num_bytes {
            head_bytes = num_bytes;
        }
        // HERE
        for i in 0..head_bytes {
            self.write_bits(bytes[i as usize] as u32, 8);
        }

        // println!("WROTE HEAD: {:#034b}", self.buffer[(self.word_index - 1) as usize]);

        if head_bytes == num_bytes {
            return true;
        }

        self.flush();
        assert!(self.get_align_bits() == 0);

        // -- Writing in words at a time --

        let num_words: u32 = (num_bytes - head_bytes) / 4;
        if num_words > 0 {
            assert!((self.bits_written % 32) == 0);
            unsafe {
                // COPY ALL THE WORDS at once into buffer.
                let dest_ptr = self.buffer.as_mut_ptr().add(self.word_index as usize);
                let src_ptr = bytes.as_ptr().add(head_bytes as usize) as *const u32;
                std::ptr::copy_nonoverlapping(src_ptr, dest_ptr, (num_words * 4) as usize);
            }
            self.bits_written += num_words * 32;
            self.word_index += num_words;
            self.scratch = 0;
        }

        assert!(self.get_align_bits() == 0);

        // -- Writing tailing word --

        let tail_bytes_start = head_bytes + num_words * 4;
        let tail_bytes = num_bytes - tail_bytes_start;
        assert!(tail_bytes < 4);
        for i in 0..tail_bytes {
            self.write_bits(bytes[(tail_bytes_start + i) as usize] as u32, 8);
        }

        assert!(self.get_align_bits() == 0);
        assert!(head_bytes + num_words * 4 + tail_bytes == num_bytes);

        return true;
    }

    pub fn get_bits_written(&mut self) -> u32 {
        return self.bits_written;
    }

    pub fn get_bytes_written(&mut self) -> u32 {
        // Add the seven and divide to round up.
        return (self.bits_written + 7) / 8;
    }

    pub fn get_align_bits(&self) -> u32 {
        return (8 - (self.bits_written % 8)) % 8;
    }
}

pub struct BitReader<'a> {
    buffer: &'a Vec<u32>,
    scratch: u64,
    scratch_bits: u32,
    num_bits: u32,
    pub num_bits_read: u32,
    word_index: u32,
}

impl BitReader<'_> {
    pub fn new(buffer: &mut Vec<u32>, bytes: u32) -> BitReader {
        assert!(buffer.len() % 4 == 0, "bytes must be a multiple of 4");
        return BitReader {
            scratch: 0,
            scratch_bits: 0,
            num_bits: bytes * 8,
            num_bits_read: 0,
            word_index: 0,
            buffer,
        };
    }

    pub fn would_read_past_end(&self, bits: u32) -> bool {
        // Returns whether or not reading 'bits' bits would overflow the available bits to read
        return self.num_bits_read + bits > self.num_bits;
    }

    // Align self.bits_read to the nearest byte
    pub fn read_align(&mut self) -> bool {
        let remainder_bits = self.num_bits_read % 8;

        if remainder_bits != 0 {
            let val = self.read_bits(8 - remainder_bits);

            if (self.num_bits_read % 8) != 0 {
                return false;
            }
            if val != 0 {
                return false;
            }
        }

        return true;
    }

    pub fn read_bits(&mut self, bits: u32) -> u32 {
        assert!(bits > 0); // Read at least one bit
        assert!(bits <= 32); // Read a maximum of 32 bits

        // Only read up to number of bits
        assert!(self.num_bits_read + bits <= self.num_bits);

        self.num_bits_read += bits;

        // Check scratch_bits is in a valid range for a 64 bit number
        assert!(self.scratch_bits <= 64);

        // If there aren't enough bits in scratch to read off the specified amount..
        if self.scratch_bits < bits {
            // Read a word from buffer into scratch, shifted left by bits in scratch
            self.scratch |= (self.buffer[self.word_index as usize] as u64) << self.scratch_bits;
            self.scratch_bits += 32;
            self.word_index += 1;
        }

        // Check that theres enough bits in scratch to read specified amount
        assert!(self.scratch_bits >= bits);

        // Copy 'bits' number of bits from scratch into output variable
        let mask = u64::pow(2, bits) - 1;
        let output: u32 = (self.scratch & mask) as u32;

        // Shift scratch right by number of bits read
        self.scratch >>= bits;
        // Subtract number of bits read from scratch_bits
        self.scratch_bits -= bits;

        output
    }

    pub fn read_bytes(&mut self, bytes: &mut [u8], num_bytes: u32) {
        println!("reading bytes: {:?} {:?}", num_bytes, bytes);

        // Check we're aligned to byte
        assert!(self.get_align_bits() == 0);

        // Check we have enough bits in buffer to actually read out num_bytes
        assert!(self.num_bits_read + num_bytes * 8 <= self.num_bits);

        // Double check we're byte aligned
        assert!(
            (self.num_bits_read % 32) == 0
                || (self.num_bits_read % 32) == 8
                || (self.num_bits_read % 32) == 16
                || (self.num_bits_read % 32) == 24
        );

        // How many bytes avail in current word
        let mut head_bytes = (4 - (self.num_bits_read % 32) / 8) % 4;

        // -- Read head --

        if head_bytes > num_bytes {
            head_bytes = num_bytes;
        }
        for n in 0..head_bytes {
            bytes[n as usize] = self.read_bits(8) as u8;
        }
        if head_bytes == num_bytes {
            return;
        }

        assert!(self.get_align_bits() == 0);

        let num_words = (num_bytes - head_bytes) / 4;

        if num_words > 0 {
            println!("READING WORDS");

            assert!((self.num_bits_read % 32) == 0);
            unsafe {
                let src_ptr = self.buffer.as_ptr().add(self.word_index as usize);
                let dest_ptr = bytes.as_ptr().add(head_bytes as usize) as *mut u32;
                std::ptr::copy_nonoverlapping(src_ptr, dest_ptr, (num_words * 4) as usize);
            }

            self.num_bits_read += num_words * 32;
            self.word_index += num_words;
            self.scratch_bits = 0;
        }

        // do tails tuff
        /*
                   assert( GetAlignBits() == 0 );
                   assert( m_bitsRead + bytes * 8 <= m_numBits );
                   assert( ( m_bitsRead % 32 ) == 0 || ( m_bitsRead % 32 ) == 8 || ( m_bitsRead % 32 ) == 16 || ( m_bitsRead % 32 ) == 24 );

                   int headBytes = ( 4 - ( m_bitsRead % 32 ) / 8 ) % 4;
                   if ( headBytes > bytes )
                       headBytes = bytes;
                   for ( int i = 0; i < headBytes; ++i )
                       data[i] = (uint8_t) ReadBits( 8 );
                   if ( headBytes == bytes )
                       return;

                   assert( GetAlignBits() == 0 );

                   int numWords = ( bytes - headBytes ) / 4;
                   if ( numWords > 0 )
                   {
                       assert( ( m_bitsRead % 32 ) == 0 );
                       memcpy( data + headBytes, &m_data[m_wordIndex], numWords * 4 );
                       m_bitsRead += numWords * 32;
                       m_wordIndex += numWords;
                       m_scratchBits = 0;
                   }

                   assert( GetAlignBits() == 0 );

                   int tailStart = headBytes + numWords * 4;
                   int tailBytes = bytes - tailStart;
                   assert( tailBytes >= 0 && tailBytes < 4 );
                   for ( int i = 0; i < tailBytes; ++i )
                       data[tailStart+i] = (uint8_t) ReadBits( 8 );

                   assert( GetAlignBits() == 0 );

                   assert( headBytes + numWords * 4 + tailBytes == bytes );
        */
    }

    pub fn get_bits_read(&self) -> u32 {
        self.num_bits_read
    }

    pub fn get_bits_remaining(&self) -> u32 {
        return self.num_bits - self.num_bits_read;
    }

    pub fn get_align_bits(&self) -> u32 {
        return (8 - (self.num_bits_read % 8)) % 8;
    }
}

mod tests {
    use super::{BitReader, BitWriter};

    #[test]
    fn test_bitpacker() {
        let buffer_size: usize = 256;

        let mut buffer: Vec<u32> = vec![0; buffer_size];
        let bits_written: u32;
        let bytes_written: u32;

        {
            let mut writer = BitWriter::new(&mut buffer, buffer_size);
            assert!(writer.get_bits_written() == 0);
            assert!(writer.get_bytes_written() == 0);
            // check( writer.GetTotalBytes() == BufferSize );
            // check( writer.GetBitsAvailable() == BufferSize * 8 );
            // check( writer.GetData() == buffer );

            writer.write_bits(0, 1);
            writer.write_bits(1, 1);
            writer.write_bits(10, 8);
            writer.write_bits(255, 8);
            writer.write_bits(1000, 10);
            writer.write_bits(50000, 16);
            writer.write_bits(9999999, 32);
            writer.flush();

            bits_written = 1 + 1 + 8 + 8 + 10 + 16 + 32; // 76
            bytes_written = writer.get_bytes_written();

            assert_eq!(10, bytes_written);
            assert_eq!(bits_written, writer.get_bits_written());
            // check( writer.GetTotalBytes() == BufferSize );
            // check( writer.GetBitsAvailable() == BufferSize * 8 - bitsWritten );
        }

        {
            let mut reader = BitReader::new(&mut buffer, bytes_written);

            assert_eq!(reader.get_bits_read(), 0);
            assert_eq!(reader.get_bits_remaining(), bytes_written * 8);

            let a = reader.read_bits(1);
            let b = reader.read_bits(1);
            let c = reader.read_bits(8);
            let d = reader.read_bits(8);
            let e = reader.read_bits(10);
            let f = reader.read_bits(16);
            let g = reader.read_bits(32);

            assert_eq!(a, 0);
            assert_eq!(b, 1);
            assert_eq!(c, 10);
            assert_eq!(d, 255);
            assert_eq!(e, 1000);
            assert_eq!(f, 50000);
            assert_eq!(g, 9999999);

            assert_eq!(reader.get_bits_read(), bits_written);
            // Check that the bits remaining is equal to the padding remaining in the bytes written
            assert_eq!(
                reader.get_bits_remaining(),
                bytes_written * 8 - bits_written
            );
        }
    }
}
