use num_traits::clamp;
use vector3d::Vector3d;
use rand::Rng;

use crate::{bits_required, assert_expr};

use self::{packet::Packet, streams::{Stream, WriteStream}};

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

pub fn serialize_string_internal(stream: &mut dyn Stream, string: &mut String) -> bool {

    let mut length: i32 = 0;
    if stream.is_writing() {
        length = string.len() as i32;
    }

    serialize_int_macro(stream, &mut length, 0, string.len() as i32);
    unsafe {
        serialize_bytes_macro(stream, string.as_bytes_mut(), length as u32);
    }

    true
}

pub fn serialize_int_macro(stream: &mut dyn Stream, value: &mut i32, min: i32, max: i32) -> bool {
    assert!(min < max);
    let mut val: i32 = 0;

    if stream.is_writing() {
        assert!(*value >= min);
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

pub fn serialize_string_macro(stream: &mut dyn Stream, string: &mut String) -> bool {
    return serialize_string_internal(stream, string);
}

pub fn serialize_bytes_macro(stream: &mut dyn Stream, bytes: &mut [u8], num_bytes: u32) -> bool {
    return serialize_bytes_internal(stream, bytes, num_bytes)
}



#[derive(Default)]
struct TestData {
    max_items: i32,
    max_int: i32,
    min_int: i32,
    items: Vec<i32>,
    num_items: i32,
    bytes: Vec<u8>,

    test_int_a: i32,
    test_int_b: i32,
    test_int_c: i32,
    test_int_d: i32,
    test_int_e: i32,
    test_int_f: i32,
    test_bool: bool,

    test_float: f32,
    test_u64: u64,
    test_string: String
}

struct TestObject {
    data: TestData
}

impl TestObject {
    pub fn new() -> TestObject {

        let mut data = TestData::default();

        data.max_items = 11;
        data.max_int = 10;
        data.min_int = -10;
        data.items = vec![0; data.max_items as usize];
        data.bytes = vec![0; 17];
    
        data.test_int_a = 1;
        data.test_int_b = -2;
        data.test_int_c = 150;
        data.test_int_d = 55;
        data.test_int_e = 255;
        data.test_int_f = 127;
        data.test_bool = true;
    
        data.num_items = data.max_items / 2;
        for i in 0..data.num_items {
            data.items[i as usize] = (i as i32) + 10;
        }
    
        data.test_float = 3.1315926;
        data.test_u64 = 0x1234567898765432;
    
        let mut rng = rand::thread_rng();
        for i in 0..data.bytes.len() {
            data.bytes[i] = rng.gen::<u8>() % 255;
        }
    
        data.test_string = String::from("Hello world!");

        return TestObject { data };
    }

    pub fn serialize(&mut self, stream: &mut dyn Stream) -> bool {
        serialize_int_macro(stream, &mut self.data.test_int_a, self.data.min_int, self.data.max_int);
        serialize_int_macro(stream, &mut self.data.test_int_b, self.data.min_int, self.data.max_int);
        serialize_int_macro(stream, &mut self.data.test_int_c, -100, 10000);
        
        // serialize_bits( stream, data.d, 6 );
        // serialize_bits( stream, data.e, 8 );
        // serialize_bits( stream, data.f, 7 );
        
        // serialize_align( stream );
        
        // serialize_bool( stream, data.g );

        // serialize_check( stream, "test object serialize check" );

        serialize_int_macro(stream, &mut self.data.num_items, 0, self.data.max_items - 1);
        for i in 0..self.data.num_items {
            // serialize_bits( stream, data.items[i], 8 );
        }

        serialize_float_macro(stream, &mut self.data.test_float);

        // serialize_uint64( stream, data.uint64_value );

        let num_bytes = self.data.bytes.len() as u32;
        serialize_bytes_macro(stream, &mut self.data.bytes, num_bytes);

        let str_len = self.data.test_string.len() as u32;
        serialize_string_macro(stream, &mut self.data.test_string);

        // serialize_check( stream, "end of test object" );

        return true;
    }
}

#[test]
fn test_serialization() {
    let mut obj = TestObject::new();
    let mut buffer = vec![0; 100];
    {
        let mut write_stream = WriteStream::new(&mut buffer);
        obj.serialize(&mut write_stream);
    }
}