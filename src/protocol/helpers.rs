use super::constants::{Buffer, ProtocolError};
use std::hash::{Hash, Hasher};

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

/** TODO */
pub fn hash_string(input: &mut String) -> u32 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    input.hash(&mut hasher);
    let hash = hasher.finish();
    return hash as u32;
}

/** TODO */
pub fn calc_packet_crc32(buffer: &Buffer, protocol_id: u32) -> u32 {
    let protocol_bytes_temp = protocol_id.to_le_bytes();
    let protocol_bytes = protocol_bytes_temp.as_slice();
    let mut crc_bytes: Vec<u8> = vec![];
    crc_bytes.extend_from_slice(protocol_bytes);
    crc_bytes.extend_from_slice(buffer);
    return crc32fast::hash(&crc_bytes);
}
