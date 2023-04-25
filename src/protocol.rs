pub mod bitpacker;
pub mod examples;
pub mod macros;
pub mod packets;
pub mod serialization;
pub mod streams;

#[derive(Copy, Clone, Debug)]
pub enum ProtocolError {
    None,
    StreamOverflow,
}

/** Prints out text representation of ProtocolError enum */
pub fn get_error_string(error: ProtocolError) {
    match error {
        ProtocolError::None => {
            println!("No error")
        }
        ProtocolError::StreamOverflow => {
            println!("Stream overflow")
        }
    }
}
