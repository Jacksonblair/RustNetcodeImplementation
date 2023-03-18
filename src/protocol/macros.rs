use super::streams::Stream;

/* c++ style assert */
#[macro_export]
macro_rules! assert_expr {
    ($cond:expr) => {
        if ($cond) == false {
            return false;
        }
    };
}

/*
    Macro for calculating number of bits required for a 32 bit value.
*/
#[macro_export]
macro_rules! bits_required {
    ($min:expr,$max:expr) => {
        if $min == $max {
            let out: u32 = 0;
            out
        } else {
            let val = $max - $min;
            let a = val | (val >> 1);
            let b = a | (a >> 2);
            let c = b | (b >> 4);
            let d = c | (c >> 8);
            let e = d | (d >> 16);
            let f = e >> 1;
            let out = f;
            out.count_ones() + 1
        }
    };
}

/*
    Macro for calculating number of bits required for a 32 bit value.

	#define serialize_int( stream, value, min, max )                    \
        do                                                              \
        {                                                               \
            assert( min < max );                                        \
            int32_t int32_value;                                        \
            if ( Stream::IsWriting )                                    \
            {                                                           \
                assert( int64_t(value) >= int64_t(min) );               \
                assert( int64_t(value) <= int64_t(max) );               \
                int32_value = (int32_t) value;                          \
            }                                                           \
            if ( !stream.SerializeInteger( int32_value, min, max ) )    \
                return false;                                           \
            if ( Stream::IsReading )                                    \
            {                                                           \
                value = int32_value;                                    \
                if ( value < min || value > max )                       \
                    return false;                                       \
            }                                                           \
        } while (0)

        */


// #[macro_export]
// macro_rules! serialise_int {
//     ($stream:expr,$value:expr,$min:expr,$max:expr) => {
//         if $min > $max {
//             println!("Min greater than max");
//             return false;
//         }

//         let mut val: i32;

//         if $stream.is_writing() {
//             if ($value as i32) < ($min as i32) {
//                 println!("Val less than min");
//                 return false;
//             }
//             if ($value as i32) > ($max as i32) {
//                 println!("Val greater than max");
//                 return false;
//             }
//             val = $value;
//         }

//         if !($stream.serialise_int(&mut val, $min, $max)) {
//             return false;
//         }

//         if $stream.is_reading() {
//             println!("READING");
//             println!("Read: {:?}", val);
//             $value = val as i32;
//             if (val as i32) < ($min as i32) || (val as i32) > ($max as i32) {
//                 return false;
//             }
//         }
//     };
// }
