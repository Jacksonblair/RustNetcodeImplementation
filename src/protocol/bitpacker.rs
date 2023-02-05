pub struct BitWriter<'a> {
    buffer: &'a mut Vec<u32>,
    scratch: u64,
    num_bits: u32,
    num_words: u32,
    bits_written: u32,
    word_index: u32,
    scratch_bits: i32
}

impl BitWriter<'_> {
    pub fn new(buffer: &mut Vec<u32>, buffer_size: u32) -> BitWriter {
        assert!(buffer_size % 4 == 0, "bytes must be a multiple of 4");
        return BitWriter {
            buffer,
            scratch: 0,
            scratch_bits: 0,
            word_index: 0,
            bits_written: 0,
            num_words: buffer_size / 4,
            num_bits: (buffer_size / 4) * 32
        }
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
        assert!(self.bits_written + num_bits <= self.num_bits);

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

    pub fn get_bits_written(&mut self) -> u32 {
        return self.bits_written;
    }

    pub fn get_bytes_written(&mut self) -> u32 {
        // Add the seven and divide to round up.
        return (self.bits_written + 7) / 8;
    }
}

pub struct BitReader<'a> {
    buffer: &'a Vec<u32>,
    scratch: u64,
    scratch_bits: u32,
    num_bits: u32,
    pub num_bits_read: u32,
    word_index: u32
}

impl BitReader<'_> {

    pub fn new(buffer: &mut Vec<u32>, bytes: u32) -> BitReader {
        assert!(buffer.len() % 4 == 0, "bytes must be a multiple of 4");
        return BitReader {
            scratch: 0,
            scratch_bits: 0,
            num_bits: bytes * 8,
            num_bits_read: 0,
            word_index: 0, buffer
        }
    }

    pub fn would_read_past_end(&self, bits: u32) -> bool {
        // Returns whether or not reading 'bits' bits would overflow the available bits to read
        return self.num_bits_read + bits > self.num_bits;
    }

    pub fn read_bits(&mut self, bits: u32) -> u32 {
        assert!(bits > 0);  // Read at least one bit
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

    pub fn get_bits_read(&self) -> u32 {
        self.num_bits_read
    }

    pub fn get_bits_remaining(&self) -> u32 {
        return self.num_bits - self.num_bits_read;
    }

}


#[test]
fn test_bitpacker()
{
    let buffer_size: u32 = 256;

    // Hypothetical local buffer
    let mut buffer: Vec<u32> = vec![0; buffer_size as usize];
    let bits_written: u32;
    let bytes_written: u32;

    {
        let mut writer = BitWriter::new(&mut buffer, buffer_size);
        assert!(writer.get_bits_written() == 0);
        assert!(writer.get_bytes_written() == 0);
        // check( writer.GetTotalBytes() == BufferSize );
        // check( writer.GetBitsAvailable() == BufferSize * 8 );
        // check( writer.GetData() == buffer );

        writer.write_bits( 0, 1 );
        writer.write_bits( 1, 1 );
        writer.write_bits( 10, 8 );
        writer.write_bits( 255, 8 );
        writer.write_bits( 1000, 10 );
        writer.write_bits( 50000, 16 );
        writer.write_bits( 9999999, 32 );
        writer.flush();

        bits_written = 1 + 1 + 8 + 8 + 10 + 16 + 32; // 76
        bytes_written = writer.get_bytes_written();

        assert_eq!(10, bytes_written);
        assert_eq!(bits_written, writer.get_bits_written());
        // check( writer.GetTotalBytes() == BufferSize );
        // check( writer.GetBitsAvailable() == BufferSize * 8 - bitsWritten );
    }

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
    assert_eq!(reader.get_bits_remaining(), bytes_written * 8 - bits_written);

}