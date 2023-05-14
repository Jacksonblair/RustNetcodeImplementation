use super::constants::Buffer;
use crate::protocol::bitpacker::{bit_reader::BitReader, bit_writer::BitWriter};

pub mod bit_reader;
pub mod bit_writer;

#[test]
fn test_bitpacker() {
    let buffer_size: usize = 256;
    let mut buffer: Buffer = vec![0; buffer_size];
    let bits_written: u32;
    let bytes_written: u32;

    {
        let mut writer = BitWriter::new(&mut buffer, buffer_size);
        assert!(writer.get_bits_written() == 0);
        assert!(writer.get_bytes_written() == 0);
        assert!(writer.get_total_bytes() == buffer_size as u32);
        assert!(writer.get_bits_available() == (buffer_size as u32) * 8);

        writer.write_bits(0, 1);
        writer.write_bits(1, 1);
        writer.write_bits(10, 8);
        writer.write_bits(255, 8);
        writer.write_bits(1000, 10);
        writer.write_bits(50000, 16);
        writer.write_bits(9999999, 32);
        writer.write_align(); // Write align before writing bytes

        let mut bytes: Vec<u8> = vec![5, 20, 255];
        writer.write_bytes(&mut bytes, 3);
        writer.flush();

        // All values + padding + bytes
        bits_written = 1 + 1 + 8 + 8 + 10 + 16 + 32 + 4 + (bytes.len() as u32 * 8); // 76
        bytes_written = writer.get_bytes_written();

        assert_eq!(bits_written / 8, bytes_written);
        assert_eq!(bits_written, writer.get_bits_written());
        assert!(writer.get_total_bytes() == buffer_size as u32);
        assert!(writer.get_bits_available() == (buffer_size as u32) * 8 - bits_written);
    }

    {
        let mut reader = BitReader::new(&mut buffer, bytes_written as usize);

        assert_eq!(reader.get_bits_read(), 0);
        assert_eq!(reader.get_bits_remaining(), bytes_written * 8);

        let a = reader.read_bits(1);
        let b = reader.read_bits(1);
        let c = reader.read_bits(8);
        let d = reader.read_bits(8);
        let e = reader.read_bits(10);
        let f = reader.read_bits(16);
        let g = reader.read_bits(32);
        reader.read_align();

        let mut bytes: Vec<u8> = vec![0; 3];
        reader.read_bytes(&mut bytes, 3);

        assert_eq!(a, 0);
        assert_eq!(b, 1);
        assert_eq!(c, 10);
        assert_eq!(d, 255);
        assert_eq!(e, 1000);
        assert_eq!(f, 50000);
        assert_eq!(g, 9999999);
        assert_eq!(bytes[0], 5);
        assert_eq!(bytes[1], 20);
        assert_eq!(bytes[2], 255);

        assert_eq!(reader.get_bits_read(), bits_written);
        // Check that the bits remaining is equal to the padding remaining in the bytes written
        assert_eq!(
            reader.get_bits_remaining(),
            bytes_written * 8 - bits_written
        );
    }
}
