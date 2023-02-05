use num_traits::clamp;

use crate::{MAX_PACKET_SIZE, bits_required};

use self::{packet::Packet, streams::{Stream, WriteStream, ReadStream}};

pub mod packet;
pub mod streams;
pub mod macros;
pub mod bitpacker;

pub fn write_packet(packet: Packet, buffer: Vec<u8>, max_packet_size: usize) {

    // Create a writestream

    // add prefix bytes

    // add crc32

    // serialize packet type

    // serialize internal (this is where the data is written)
        /*
            Packet_n {
                some_game_variable

                serialize_internal(stream) {
                    write_variable(stream, variable)
                }
            }
        */


    // flush stream

}

pub fn serialize_float(stream: &mut dyn Stream, value: &mut f32) -> bool {

    // Convert float to an integer representation
    let mut as_int = value.to_bits();
    let result = stream.serialize_bits(&mut as_int, 32);

    if stream.is_reading() {
        // Convert integer representation to a float
        *value = f32::from_bits(as_int);
    }

    return result;
}

pub fn serialize_compressed_float(stream: &mut dyn Stream, value: &mut f32, min: f32, max: f32, precision: f32) -> bool {
    /*
        To compress a float in a range to a specified precision...
        - Divide the delta (min 0 - max 11 == 11) by the precision (ex. 11 / 0.01 == 1100) to get the number of required values
        - Normalise the value between 0 and 1 (0.454545...)
        - Multiply the normalised value by the max integer value (0.4545... * 1100) and add 0.05 == 500 ish
        - Send that value (floored to an integer)

        To decompress
        - Divide return value by max integer value (500ish / 1100 == 0.4545...)
        - Multiply that by the delta, and add the minimum (0.4545... * 11 + 0) = 500 ish
    */

    let delta: f32 = max - min;
    let num_values: f32 = delta / precision;
    let max_integer_value = f32::ceil(num_values) as u32;
    let bits = bits_required!(0, max_integer_value);
    let mut integer_value: u32 = 0;

    if stream.is_writing() {
        let normalised_value = clamp((*value - min) / delta, 0.0, 1.0);
        integer_value = f32::floor(normalised_value * max_integer_value as f32 + 0.05) as u32;
    }

    if !stream.serialize_bits(&mut integer_value, bits) {
        false;
    }

    if stream.is_reading() {
        let normalised_value = integer_value as f32 / max_integer_value as f32;
        *value = normalised_value * delta + min;
    }

    return true;
}

// TODO: Turn into a macro
pub fn serialize_int(stream: &mut dyn Stream, value: &mut i32, min: i32, max: i32) -> bool {
    if min > max {
        println!("Min greater than max");
        false;
    }

    let mut val: i32 = 0;

    if stream.is_writing() {
        if *value < min {
            println!("Val less than min");
            false;
        }
        if *value > max {
            println!("Val greater than max");
            false;
        }
        val = *value;
    }

    if !stream.serialise_int(&mut val, min, max) {
        false;
    }

    if stream.is_reading() {
        println!("READING");
        println!("Read: {:?}", val);
        *value = val;
        if val < min || val > max {
            return false;
        }
    }

    return true;
}



#[test]
fn test_serialization() {
    {
        let mut write = 20;
        let mut read = 0;
        let mut buffer: Vec<u32> = vec![0; 100];

        {
            let mut write_stream = WriteStream::new(&mut buffer);
            serialize_int(&mut write_stream, &mut write, 0, 40);
            write_stream.writer.flush();
        }

        let mut read_stream = ReadStream::new(&mut buffer);
        serialize_int(&mut read_stream, &mut read, 0, 40);
        assert_eq!(read, 20);
    }

    {
        let mut write: f32 = 49.3;
        let mut read: f32 = 0.;
        let mut buffer: Vec<u32> = vec![0; 100];

        {
            let mut write_stream = WriteStream::new(&mut buffer);
            serialize_float(&mut write_stream, &mut write);
            write_stream.writer.flush();
        }

        let mut read_stream = ReadStream::new(&mut buffer);
        serialize_float(&mut read_stream, &mut read);
        assert_eq!(read, 49.3);
    }

    {
        let mut write: f32 = 5.54;
        let mut read: f32 = 0.;
        let mut buffer: Vec<u32> = vec![0; 100];

        {
            let mut write_stream = WriteStream::new(&mut buffer);
            serialize_compressed_float(&mut write_stream, &mut write, 0., 11., 0.5);
            write_stream.writer.flush();
        }

        let mut read_stream = ReadStream::new(&mut buffer);
        serialize_compressed_float(&mut read_stream, &mut read, 0., 11., 0.5);
        assert_eq!(read, 5.5);
    }

}