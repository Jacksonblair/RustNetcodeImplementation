use crate::protocol::constants::Buffer;

pub struct BitWriter<'a> {
    buffer: &'a mut Buffer, // Pointer to buffer
    scratch: u64,           // 64 bit scratch buffer
    num_bits: u32,          // Bits in buffer
    num_words: u32,         // Words in buffer
    bits_written: u32,      // Number of bits written to buffer
    word_index: u32,        // Current word index
    scratch_bits: i32,      // Number of bits in scratch buffer
}

impl BitWriter<'_> {
    pub fn new(buffer: &mut Buffer, buffer_size: usize) -> BitWriter {
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

    /** Aligns bitwriter position to next byte */
    pub fn write_align(&mut self) -> bool {
        let remainder_bits = self.bits_written % 8;
        if remainder_bits != 0 {
            self.write_bits(0, 8 - remainder_bits);
            return (self.bits_written % 8) == 0;
        }
        return true;
    }

    /** Flush remaining bits in scratch into buffer */
    pub fn flush(&mut self) {
        if self.scratch_bits != 0 {
            assert!(self.word_index < self.num_words);

            // Copy scratch into buffer
            let bytes = (self.scratch as u32).to_le_bytes();
            for i in 0..4 {
                self.buffer[((self.word_index * 4) as usize) + i] = bytes[i];
            }

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
            // println!("Scratch full: {:#066b}", self.scratch);
            // println!("Adding to buffer: {:#034b}", self.scratch);
            // println!("Buffer: {:#034b}", self.buffer[self.word_index as usize]);
            // println!("Scratch bits: {:?}", self.scratch_bits);

            // Copy scratch into buffer
            unsafe {
                let src = (self.scratch as u32).to_le_bytes().as_ptr();
                let dest = self.buffer.as_mut_ptr().add((self.word_index * 4) as usize);
                std::ptr::copy_nonoverlapping(src, dest, 4);
            }

            self.scratch >>= 32;
            self.scratch_bits -= 32;
            self.word_index += 1;
        }

        self.bits_written += num_bits;
    }

    pub fn write_bytes(&mut self, bytes: &mut Vec<u8>, num_bytes: u32) -> bool {
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
        for i in 0..head_bytes {
            self.write_bits(bytes[i as usize] as u32, 8);
        }

        // println!(
        //     "WROTE HEAD: {:#034b}",
        //     self.buffer[(self.word_index - 1) as usize]
        // );

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
                let dest_ptr = self.buffer.as_mut_ptr().add((self.word_index * 4) as usize);
                let src_ptr = bytes.as_ptr().add(head_bytes as usize);
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

    pub fn get_total_bytes(&mut self) -> u32 {
        self.num_words * 4
    }

    pub fn get_bits_available(&mut self) -> u32 {
        return self.num_bits - self.bits_written;
    }

    pub fn get_align_bits(&self) -> u32 {
        return (8 - (self.bits_written % 8)) % 8;
    }
}
