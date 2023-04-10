use num_traits::clamp;
pub mod bitpacker;
pub mod macros;
pub mod packets;
pub mod serialization;
pub mod streams;

type Buffer = Vec<u32>;
