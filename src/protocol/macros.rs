macro_rules! some_macro {
    ($cond:expr $(,)?) => {
        if !($cond) {
            return
        }
    }
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
            let out: u32 = f;
            out.count_ones() + 1
        }
    };
}


/*
    Macro for calculating number of bits required for a 32 bit value.
*/
#[macro_export]
macro_rules! serialise_int {
    ($stream:expr,$value:expr,$min:expr,$max:expr) => {
        if $min > $max {
            println!("Min greater than max");
            return false;
        }

        let mut val: u32 = 0;
        if $stream.is_writing() {
            if $value < $min {
                println!("Val less than min");
                return false;
            }
            if ($value > $max) {
                println!("Val greater than max");
                return false;
            }
            val = $value;
        }

        if !($stream.serialise_int(&mut val, $min, $max)) {
            return false;
        }

        if $stream.is_reading() {
            println!("READING");
            println!("Read: {:?}", val);
            $value = val;
            if val < $min || val > $max {
                return false;
            }
        }
    };
}
