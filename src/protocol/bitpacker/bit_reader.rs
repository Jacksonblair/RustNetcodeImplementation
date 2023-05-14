use crate::protocol::constants::Buffer;

fn print_word(bytes: &Vec<u8>, idx: usize) {
    println!(
        "BUFFER: {:#010b} {:#010b} {:#010b} {:#010b}",
        bytes[idx],
        bytes[idx + 1],
        bytes[idx + 2],
        bytes[idx + 3]
    );
}

pub struct BitReader<'a> {
    buffer: &'a Buffer,     // Pointer to buffer
    scratch: u64,           // 64 bit scratch buffer
    pub scratch_bits: u32,  // Number of bits in scratch buffer
    num_bits: u32,          // Bits in buffer
    pub num_bits_read: u32, // Number of bits read from buffer
    pub word_index: u32,    // Current word index
}

impl BitReader<'_> {
    pub fn new(buffer: &mut Buffer, bytes: usize) -> BitReader {
        assert!(buffer.len() % 4 == 0, "bytes must be a multiple of 4");
        return BitReader {
            scratch: 0,
            scratch_bits: 0,
            num_bits: (bytes as u32) * 8,
            num_bits_read: 0,
            word_index: 0,
            buffer,
        };
    }

    pub fn get_word_index(&self) -> usize {
        return self.word_index as usize;
    }

    pub fn print_word(&self, idx: usize) {
        print_word(self.buffer, idx);
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
            // TODO: Can i do this without
            // Read a word from buffer into scratch, shifted left by bits in scratch
            let word_index = (self.word_index * 4) as usize;
            let word_bytes = [
                self.buffer[word_index],
                self.buffer[word_index + 1],
                self.buffer[word_index + 2],
                self.buffer[word_index + 3],
            ];
            let word = u32::from_le_bytes(word_bytes);
            self.scratch |= (word as u64) << self.scratch_bits;
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

    pub fn read_bytes(&mut self, bytes: &mut Vec<u8>, num_bytes: u32) {
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

        // -- Reading leading word --

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

        // -- Reading words at a time --

        let num_words = (num_bytes - head_bytes) / 4;
        if num_words > 0 {
            assert!((self.num_bits_read % 32) == 0);
            unsafe {
                let src_ptr = self.buffer.as_ptr().add((self.word_index * 4) as usize);
                let dest_ptr = bytes.as_mut_ptr().add(head_bytes as usize);
                std::ptr::copy_nonoverlapping(src_ptr, dest_ptr, (num_words * 4) as usize);
            }

            self.num_bits_read += num_words * 32;
            self.word_index += num_words;
            self.scratch_bits = 0;
        }

        // -- Reading tail --
        let tail_start = head_bytes + num_words * 4;
        let tail_bytes = num_bytes - tail_start;
        assert!(tail_bytes < 4);

        for i in 0..tail_bytes {
            bytes[(tail_start + i) as usize] = self.read_bits(8) as u8;
        }

        assert!(self.get_align_bits() == 0);
        assert!(head_bytes + num_words * 4 + tail_bytes == num_bytes);
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
