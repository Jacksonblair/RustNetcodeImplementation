use num_traits::clamp;
use vector3d::Vector3d;

use crate::{bits_required,assert_expr};

use self::{packet::Packet, streams::{Stream, WriteStream, ReadStream}};

pub mod packet;
pub mod streams;
pub mod macros;
pub mod bitpacker;

pub fn serialize_float_internal(stream: &mut dyn Stream, value: &mut f32) -> bool {

    // Convert float to an integer representation
    let mut as_int = value.to_bits();
    let result = stream.serialize_bits(&mut as_int, 32);

    if stream.is_reading() {
        // Convert integer representation to a float
        *value = f32::from_bits(as_int);
    }

    return result;
}

pub fn serialize_compressed_float_internal(stream: &mut dyn Stream, value: &mut f32, min: f32, max: f32, precision: f32) -> bool {
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

pub fn serialize_vector_internal(stream: &mut dyn Stream, vector: &mut Vector3d<f32>) -> bool {
    
    let mut values = vec![0.; 3];

    if stream.is_writing() {
        values[0] = vector.x;
        values[1] = vector.y;
        values[2] = vector.z;
    }
    serialize_float_macro(stream, &mut values[0]);
    serialize_float_macro(stream, &mut values[1]);
    serialize_float_macro(stream, &mut values[2]);
    if stream.is_reading() {
        vector.x = values[0];
        vector.y = values[1];
        vector.z = values[2];
    }

    return true
}

pub fn serialize_compressed_vector_internal(stream: &mut dyn Stream, vector: &mut Vector3d<f32>, min: f32, max: f32, precision: f32) -> bool {

    let mut values = vec![0.; 3];

    if stream.is_writing() {
        values[0] = vector.x;
        values[1] = vector.y;
        values[2] = vector.z;
    }
    serialize_compressed_float_macro(stream, &mut values[0], min, max, precision);
    serialize_compressed_float_macro(stream, &mut values[1], min, max, precision);
    serialize_compressed_float_macro(stream, &mut values[2], min, max, precision);

    if stream.is_reading() {
        vector.x = values[0];
        vector.y = values[1];
        vector.z = values[2];
    }

    return true;
}

pub fn serialize_bytes_internal(stream: &mut dyn Stream, bytes: &mut [u8], num_bytes: u32) -> bool {
    return stream.serialize_bytes(bytes, num_bytes);
}

pub fn serialize_string_internal(stream: &mut dyn Stream, string: &mut String, buffer_size: u32) -> bool {

    let mut length: i32 = 0;
    if stream.is_writing() {
        length = string.len() as i32;
        assert!(length < (buffer_size - 1) as i32);
    }

    // When im writing, the vec is the right size
    // When im reading, the string is empty so the vec is too.
    // I need to have a vector long enough to hold the string. 
    let mut bytes = string.as_bytes().to_vec();

    serialize_int_macro(stream, &mut length, 0, (buffer_size - 1) as i32);
    serialize_bytes_macro(stream, &mut bytes, length as u32);

    if stream.is_reading() {
        println!("READING");
        *string = std::str::from_utf8(&bytes).unwrap().to_string();
        // READ VALUE BACK INTO STRING
        // string[0];
        // string[length] = "\0";
    }

    true
}


// TODO: Turn into a macro
pub fn serialize_int_macro(stream: &mut dyn Stream, value: &mut i32, min: i32, max: i32) -> bool {
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

        *value = val;
        if val < min || val > max {
            return false;
        }

        // println!("READ OFF OF BUFFER: {:?}", val);
        // println!("READ OFF OF BUFFER: {:?}", *value);
    }

    return true;
}

// TODO: Turn into a macro
pub fn serialize_float_macro(stream: &mut dyn Stream, value: &mut f32) -> bool {
    if !serialize_float_internal(stream, value) {
        return false
    }
    true
}

// TODO: Turn into a macro
pub fn serialize_compressed_float_macro(stream: &mut dyn Stream, value: &mut f32, min: f32, max: f32, precision: f32) -> bool {
    if !serialize_compressed_float_internal(stream, value, min, max, precision) {
        return false
    }
    true
}

pub fn serialize_vector_macro(stream: &mut dyn Stream, vector: &mut Vector3d<f32>) -> bool {
    if !serialize_vector_internal(stream, vector) {
        false;
    }
    true
}

pub fn serialize_string_macro(stream: &mut dyn Stream, string: &mut String, buffer_size: u32) -> bool {
    return serialize_string_internal(stream, string, buffer_size);
}

pub fn serialize_bytes_macro(stream: &mut dyn Stream, bytes: &mut [u8], num_bytes: u32) -> bool {
    return serialize_bytes_internal(stream, bytes, num_bytes)
}


#[test]
fn test_serialization() {

    // {
    //     // Test serialize int
    //     let mut write = 20;
    //     let mut read = 0;
    //     let mut buffer: Vec<u32> = vec![0; 100];

    //     {
    //         let mut write_stream = WriteStream::new(&mut buffer);
    //         serialize_int_macro(&mut write_stream, &mut write, 0, 40);
    //         write_stream.writer.flush();
    //     }

    //     let mut read_stream = ReadStream::new(&mut buffer);
    //     serialize_int_macro(&mut read_stream, &mut read, 0, 40);
    //     assert_eq!(read, 20);
    // }

    // {
    //     // Test serialize float
    //     let mut write: f32 = 49.3;
    //     let mut read: f32 = 0.;
    //     let mut buffer: Vec<u32> = vec![0; 100];

    //     {
    //         let mut write_stream = WriteStream::new(&mut buffer);
    //         serialize_float_internal(&mut write_stream, &mut write);
    //         write_stream.writer.flush();
    //     }

    //     let mut read_stream = ReadStream::new(&mut buffer);
    //     serialize_float_internal(&mut read_stream, &mut read);
    //     assert_eq!(read, 49.3);
    // }

    // {
    //     // Test serialize compressed float
    //     let mut write: f32 = 5.54;
    //     let mut read: f32 = 0.;
    //     let mut buffer: Vec<u32> = vec![0; 100];

    //     {
    //         let mut write_stream = WriteStream::new(&mut buffer);
    //         serialize_compressed_float_internal(&mut write_stream, &mut write, 0., 11., 0.5);
    //         write_stream.writer.flush();
    //     }

    //     let mut read_stream = ReadStream::new(&mut buffer);
    //     serialize_compressed_float_internal(&mut read_stream, &mut read, 0., 11., 0.5);
    //     assert_eq!(read, 5.5);
    // }

    // {
    //     // Test serialize vector
    //     let mut write: Vector3d<f32> = Vector3d::new(10., 20., 30.6);
    //     let mut read: Vector3d<f32> = Vector3d::new(0., 0., 0.);
    //     let mut buffer: Vec<u32> = vec![0; 100];

    //     {
    //         let mut write_stream = WriteStream::new(&mut buffer);
    //         serialize_vector_internal(&mut write_stream, &mut write);
    //         write_stream.writer.flush();
    //     }

    //     let mut read_stream = ReadStream::new(&mut buffer);
    //     serialize_vector_internal(&mut read_stream, &mut read);
    //     assert_eq!(read.x, 10.);
    //     assert_eq!(read.y, 20.);
    //     assert_eq!(read.z, 30.6);
    // }

    // {
    //     // Test serialize compressed vector
    //     let mut write: Vector3d<f32> = Vector3d::new(10., 20., 30.9);
    //     let mut read: Vector3d<f32> = Vector3d::new(0., 0., 0.);
    //     let mut buffer: Vec<u32> = vec![0; 100];
    //     let min = 8.;
    //     let max = 40.;
    //     let precision = 0.5;

    //     {
    //         let mut write_stream = WriteStream::new(&mut buffer);
    //         serialize_compressed_vector_internal(&mut write_stream, &mut write, min, max, precision);
    //         write_stream.writer.flush();
    //     }

    //     let mut read_stream = ReadStream::new(&mut buffer);
    //     serialize_compressed_vector_internal(&mut read_stream, &mut read, min, max, precision);
        
    //     println!("{:?}", read);

    //     assert_eq!(read.x, 10.);
    //     assert_eq!(read.y, 20.);
    //     assert_eq!(read.z, 30.5);
    // }

    {
        // TODO: WHats a nicer way to make a string thats long enough to read into??

        // Test serialize string
        let mut write = String::from("hello you dingus");
        let length = write.len() as u32;
        let mut read: String = String::from("                ");
        let mut buffer: Vec<u32> = vec![0; 100];

        println!("WRITING: {:?} ", length);

        {
            let mut write_stream = WriteStream::new(&mut buffer);
            serialize_string_macro(&mut write_stream, &mut write, 100);
            write_stream.writer.flush();
        }


        let mut read_stream = ReadStream::new(&mut buffer);
        serialize_string_macro(&mut read_stream, &mut read, 100);

        println!("Read: {:?}", read);

        assert!(1 == 2);

    }

}