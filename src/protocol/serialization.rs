use num_traits::clamp;
use rand::Rng;
use vector3d::Vector3d;

use crate::bits_required;

use super::{packets::Object, streams::Stream};

pub const MAX_OBJECTS: u32 = 1024;

pub fn serialize_object(stream: &mut dyn Stream, object: &mut dyn Object) -> bool {
    // return object.serialize(stream);
    true
}

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

pub fn serialize_compressed_float_internal<T: Stream>(
    stream: &mut T,
    value: &mut f32,
    min: f32,
    max: f32,
    precision: f32,
) -> bool {
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

pub fn serialize_vector_internal<T: Stream>(stream: &mut T, vector: &mut Vector3d<f32>) -> bool {
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

    return true;
}

pub fn serialize_compressed_vector_internal<T: Stream>(
    stream: &mut T,
    vector: &mut Vector3d<f32>,
    min: f32,
    max: f32,
    precision: f32,
) -> bool {
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

pub fn serialize_bytes_internal<T: Stream>(
    stream: &mut T,
    bytes: &mut [u8],
    num_bytes: u32,
) -> bool {
    return stream.serialize_bytes(bytes, num_bytes);
}

pub fn serialize_string_internal<T: Stream>(
    stream: &mut T,
    string: &mut String,
    buffer_size: usize,
) -> bool {
    let mut length: i32 = 0;
    if stream.is_writing() {
        length = string.len() as i32;
        assert!((length as usize) < buffer_size - 1);
    }

    serialize_int_macro(stream, &mut length, 0, buffer_size as i32);

    /* We serialize the length, create a byte vec to hold. Is this a bad idea? */
    let mut bytes: Vec<u8> = vec![0; length as usize];
    let string_bytes = string.as_bytes();
    for i in 0..bytes.len() {
        bytes[i] = string_bytes[i];
    }

    serialize_bytes_macro(stream, &mut bytes, length as u32);

    if stream.is_reading() {
        *string = String::from_utf8(bytes).unwrap();
    }

    true
}

pub fn serialize_int_macro(stream: &mut dyn Stream, value: &mut i32, min: i32, max: i32) -> bool {
    assert!(min < max);
    let mut val: i32 = 0;

    if stream.is_writing() {
        assert!(
            *value >= min,
            "Value ({}) is < minimum value ({})",
            *value,
            min
        );
        assert!(*value <= max);
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
    }

    return true;
}

// TODO: Turn into a macro
pub fn serialize_float_macro(stream: &mut dyn Stream, value: &mut f32) -> bool {
    if !serialize_float_internal(stream, value) {
        return false;
    }
    true
}

// TODO: Turn into a macro
pub fn serialize_compressed_float_macro<T: Stream>(
    stream: &mut T,
    value: &mut f32,
    min: f32,
    max: f32,
    precision: f32,
) -> bool {
    if !serialize_compressed_float_internal(stream, value, min, max, precision) {
        return false;
    }
    true
}

pub fn serialize_vector_macro<T: Stream>(stream: &mut T, vector: &mut Vector3d<f32>) -> bool {
    if !serialize_vector_internal(stream, vector) {
        false;
    }
    true
}

pub fn serialize_string_macro<T: Stream>(
    stream: &mut T,
    string: &mut String,
    buffer_size: usize,
) -> bool {
    return serialize_string_internal(stream, string, buffer_size);
}

pub fn serialize_bytes_macro<T: Stream>(stream: &mut T, bytes: &mut [u8], num_bytes: u32) -> bool {
    return serialize_bytes_internal(stream, bytes, num_bytes);
}

pub fn serialize_bits_macro(stream: &mut dyn Stream, value: &mut u32, bits: u32) -> bool {
    assert!(bits > 0);
    assert!(bits <= 32);
    let mut u32_val: u32 = 0;
    if stream.is_writing() {
        u32_val = *value;
    }
    if !stream.serialize_bits(&mut u32_val, bits) {
        return false;
    }
    if stream.is_reading() {
        *value = u32_val;
    }
    true
}

pub fn serialize_bool_macro(stream: &mut dyn Stream, value: &mut bool) -> bool {
    let mut uint32_bool_value = 0;
    if stream.is_writing() {
        if *value == true {
            uint32_bool_value = 1;
        } else {
            uint32_bool_value = 0;
        }
    }
    serialize_bits_macro(stream, &mut uint32_bool_value, 1);
    if stream.is_reading() {
        if uint32_bool_value == 1 {
            *value = true;
        } else {
            *value = false;
        }
    }
    true
}

pub fn serialize_object_index_internal(
    stream: &mut dyn Stream,
    previous: &mut i32,
    current: &mut i32,
) -> bool {
    let mut difference: i32 = 0;

    /*
        Explaining this:
        - We're generally encoding the number of bits required to encode the index diff, and then the encoded index diff
        - * If we're only serializing a index diff of 1, we dont need to encode the bits required, because it can only be one
        - * If we serializing a diff that takes over 6 bits to encode (12 including the bits required), we just serialize enough bits to encode the maximum number of objects

        We write a boolean for each index to indicate the number of bits required to serialize an index, and then the actual index.
        EX. If the index takes 5 bits to serialize, we will write 5 zeroes before we write the value.
        EX. If the index takes over 6 bits to encode, we wil write 6 zeroes and then just serialize the value using the max amount of bits it could be.
    */

    if stream.is_writing() {
        assert!(*previous < *current);
        difference = (*current - *previous) as i32;
        assert!(difference > 0);
    }

    // +1 (1 bit)
    let mut plus_one: bool = false;
    if stream.is_writing() {
        plus_one = difference == 1;
    }
    serialize_bool_macro(stream, &mut plus_one);
    if plus_one {
        if stream.is_reading() {
            *current = *previous + 1;
        }
        *previous = *current;
        return true;
    }

    // [+2, 5] -> [0,3] (2 bits)
    let mut two_bits = false;
    if stream.is_writing() {
        two_bits = difference <= 5;
    }
    serialize_bool_macro(stream, &mut two_bits);
    if two_bits {
        serialize_int_macro(stream, &mut difference, 2, 5);
        if stream.is_reading() {
            *current = *previous + difference;
        }
        *previous = *current;
        return true;
    }

    // [6, 13] -> [0,7] (3 bits)
    let mut three_bits = false;
    if stream.is_writing() {
        three_bits = difference <= 13;
    }
    serialize_bool_macro(stream, &mut three_bits);
    if three_bits {
        serialize_int_macro(stream, &mut difference, 6, 13);
        if stream.is_reading() {
            *current = *previous - difference;
        }
        *previous = *current;
        return true;
    }

    // [14,29] -> [0,15] (4 bits)
    let mut four_bits = false;
    if stream.is_writing() {
        four_bits = difference <= 29;
    }
    serialize_bool_macro(stream, &mut four_bits);
    if four_bits {
        serialize_int_macro(stream, &mut difference, 14, 29);
        if stream.is_reading() {
            *current = *previous + difference;
        }
        *previous = *current;
        return true;
    }

    // [30,61] -> [0,31] (5 bits)
    let mut five_bits = false;
    if stream.is_writing() {
        five_bits = difference <= 61;
    }
    serialize_bool_macro(stream, &mut five_bits);
    if five_bits {
        serialize_int_macro(stream, &mut difference, 30, 61);
        if stream.is_reading() {
            *current = *previous + difference;
        }
        *previous = *current;
        return true;
    }

    // [62,125] -> [0,63] (6 bits)
    let mut six_bits = false;
    if stream.is_writing() {
        six_bits = difference <= 125;
    }
    serialize_bool_macro(stream, &mut six_bits);
    if six_bits {
        serialize_int_macro(stream, &mut difference, 62, 125);
        if stream.is_reading() {
            *current = *previous + difference;
        }
        *previous = *current;
        return true;
    }

    // [126, MAX_OBJECTS+1]
    serialize_int_macro(stream, &mut difference, 126, (MAX_OBJECTS + 1) as i32);

    if stream.is_reading() {
        *current = *previous + difference;
        if *current > MAX_OBJECTS as i32 {
            *current = MAX_OBJECTS as i32;
        }
    }
    *previous = *current;

    return true;
}

// TODO: Turn into a macro
pub fn read_object_index_macro(
    stream: &mut dyn Stream,
    previous: &mut i32,
    current: &mut i32,
) -> bool {
    return serialize_object_index_internal(stream, previous, current);
}

// TODO: Turn into a macro
pub fn write_object_index_macro(stream: &mut dyn Stream, previous: &mut i32, current: i32) -> bool {
    let mut temp_current: i32 = current;
    return serialize_object_index_internal(stream, previous, &mut temp_current);
}

mod tests {
    use super::*;
    use crate::{
        impl_object_for_packet,
        protocol::streams::{ReadStream, WriteStream},
    };

    #[derive(PartialEq, Debug)]
    struct TestData {
        // num_items: i32,
        // items: Vec<u32>,
        // bytes: Vec<u8>,
        test_int_a: i32,
        test_int_b: i32,
        test_int_c: i32,
        test_int_d: u32,
        test_int_e: u32,
        test_int_f: u32,
        test_bool: bool,

        test_float: f32,
        // test_u64: u64,
        test_string: String,
    }

    impl Default for TestData {
        fn default() -> Self {
            return TestData {
                // num_items: 20,
                // items: vec![0; 20],
                // bytes: vec![0; 17],
                test_int_a: 0,
                test_int_b: 0,
                test_int_c: 0,
                test_int_d: 0,
                test_int_e: 0,
                test_int_f: 0,
                test_bool: false,
                test_float: 0.0,
                // test_u64: 0,
                test_string: String::from_utf8(vec![0; 500]).unwrap(),
            };
        }
    }

    #[derive(Debug)]
    struct TestObject {
        max_items: i32,
        max_int: i32,
        min_int: i32,
        data: TestData,
    }

    impl TestObject {
        pub fn new() -> TestObject {
            return TestObject {
                max_items: 11,
                max_int: 10,
                min_int: -10,
                data: TestData::default(),
            };
        }

        pub fn init(&mut self) {
            self.data.test_int_a = 1;
            self.data.test_int_b = -2;
            self.data.test_int_c = 150;
            self.data.test_int_d = 55;
            self.data.test_int_e = 255;
            self.data.test_int_f = 127;
            self.data.test_bool = true;

            self.data.test_float = 3.1315926;
            // self.data.test_u64 = 0x1234567898765432;

            // self.data.items = vec![0; self.max_items as usize];
            // self.data.bytes = vec![0; 17];

            // self.data.num_items = self.max_items / 2;
            // for i in 0..self.data.num_items {
            //     self.data.items[i as usize] = i as u32 + 10;
            // }

            // let mut rng = rand::thread_rng();
            // for i in 0..self.data.bytes.len() {
            //     self.data.bytes[i] = rng.gen::<u8>() % 255;
            // }

            self.data.test_string = String::from("Hello world!");
        }

        pub fn serialize<T: Stream>(&mut self, stream: &mut T) -> bool {
            serialize_int_macro(
                stream,
                &mut self.data.test_int_a,
                self.min_int,
                self.max_int,
            );
            serialize_int_macro(
                stream,
                &mut self.data.test_int_b,
                self.min_int,
                self.max_int,
            );
            serialize_int_macro(stream, &mut self.data.test_int_c, -100, 10000);
            serialize_bits_macro(stream, &mut self.data.test_int_d, 6);
            serialize_bits_macro(stream, &mut self.data.test_int_e, 8);
            serialize_bits_macro(stream, &mut self.data.test_int_f, 7);

            // serialize_align(stream);

            serialize_bool_macro(stream, &mut self.data.test_bool);
            serialize_float_macro(stream, &mut self.data.test_float); // BUSTED

            serialize_string_macro(stream, &mut self.data.test_string, 100);

            // serialize_check( stream, "test object serialize check" );

            // serialize_int_macro(
            //     stream,
            //     &mut self.data.num_items,
            //     0,
            //     (self.max_items as i32) - 1,
            // );
            // for i in 0..self.data.num_items {
            //     serialize_bits_macro(stream, &mut self.data.items[i as usize], 8);
            // }

            // let num_bytes = self.data.bytes.len() as u32;
            // serialize_bytes_macro(stream, &mut self.data.bytes, num_bytes);
            // serialize_uint64(stream, data.uint64_value);
            // let str_len = self.data.test_string.len() as u32;

            // serialize_check( stream, "end of test object" );

            return true;
        }
    }

    impl_object_for_packet!(TestObject);

    #[test]
    fn test_serialization() {
        let mut write_obj = TestObject::new();
        write_obj.init();
        let mut buffer = vec![0; 100];
        let buffer_size = buffer.len();

        {
            let mut write_stream = WriteStream::new(&mut buffer, buffer_size);
            write_obj.serialize(&mut write_stream);
        }

        let mut read_stream = ReadStream::new(&mut buffer, buffer_size);
        let mut read_object = TestObject::new();
        read_object.serialize(&mut read_stream);

        assert!(read_object.data == write_obj.data);
    }
}
