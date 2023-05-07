use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

pub mod bitpacker;
pub mod examples;
pub mod macros;
pub mod packets;
pub mod serialization;
pub mod streams;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ProtocolError {
    None = 0,
    StreamOverflow = 1,
    SerializeHeaderFailed = 2,
    InvalidPacketType = 3,
    PacketTypeNotAllowed = 4,
    CreatePacketFailed = 5,
    SerializePacketFailed = 6,
    SerializeCheckFailed = 7,
}

pub type Buffer = Vec<u8>;

/*
  uint32_t calculate_crc32(const uint8_t *buffer, size_t length, uint32_t crc32)
  {
      crc32 ^= 0xFFFFFFFF;
      for (size_t i = 0; i < length; ++i)
          crc32 = (crc32 >> 8) ^ crc32_table[(crc32 ^ buffer[i]) & 0xFF];
      return crc32 ^ 0xFFFFFFFF;
  }
*/

/** Prints out text representation of ProtocolError enum */
pub fn get_error_string(error: ProtocolError) -> &'static str {
    match error {
        ProtocolError::None => return "No error",
        ProtocolError::StreamOverflow => return "Stream overflow",
        ProtocolError::SerializeHeaderFailed => return "Failed to serialize header",
        ProtocolError::CreatePacketFailed => return "Failed to create packet",
        ProtocolError::InvalidPacketType => return "Invalid packet type",
        ProtocolError::PacketTypeNotAllowed => return "Packet type not allowed",
        ProtocolError::SerializeCheckFailed => return "Serialize check failed",
        ProtocolError::SerializePacketFailed => return "Serialize packet failed",
    }
}

pub fn to_bytes(input: &[u32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(4 * input.len());

    for value in input {
        bytes.extend(&value.to_be_bytes());
    }

    bytes
}

/** TODO: FINISH ?? */
pub fn hash_string(input: &mut String) -> u32 {
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    let hash = hasher.finish();
    return hash as u32;
}

pub fn calc_packet_crc32(buffer: &Buffer, protocol_id: u32) -> u32 {
    let protocol_bytes_temp = protocol_id.to_le_bytes();
    let protocol_bytes = protocol_bytes_temp.as_slice();
    let mut crc_bytes: Vec<u8> = vec![];
    crc_bytes.extend_from_slice(protocol_bytes);
    crc_bytes.extend_from_slice(buffer);
    return crc32fast::hash(&crc_bytes);
}
