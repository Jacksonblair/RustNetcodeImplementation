use std::slice::from_raw_parts;

use crate::protocol::streams::read_stream::ReadStream;
use crate::protocol::streams::write_stream::WriteStream;
use crate::protocol::streams::Stream;

use self::object::*;
use self::packet_info::*;

use super::constants::Buffer;
use super::constants::ProtocolError;

pub mod fragment_packet;
pub mod object;
pub mod packet_buffer;
pub mod packet_data;
pub mod packet_factory;
pub mod packet_info;

/** TODO */
pub fn write_packet(
    info: &PacketInfo,
    packet: &mut dyn Packet,
    buffer: &mut Buffer,
    buffer_length: usize,
    header: Option<&mut dyn Object>,
) -> u32 {
    assert!(buffer.len() > 0);
    assert!(buffer.len() <= buffer_length);

    let num_packet_types = info.packet_factory.get_num_packet_types();
    let buffer_length = buffer.len();
    let mut stream = WriteStream::new(buffer, buffer.len());

    let mut crc_32: u32 = 0;
    // stream.SetContext(info.context);

    // Serialize prefix bytes
    for _i in 0..info.prefix_bytes {
        let mut zero: u32 = 0;
        stream.serialize_bits(&mut zero, 8);
    }

    // Serialize space for crc32, which we calculate at the end of writing the packet.
    if !info.raw_format {
        stream.serialize_bits(&mut crc_32, 32);
    }

    // Write header if there is one
    match header {
        Some(header) => {
            header.serialize_internal_w(&mut stream);
        }
        _ => (),
    }

    let mut packet_type = packet.get_packet_type() as i32;
    assert!(num_packet_types > 0);

    // If we have more than one packet type, serialize the packet type into the buffer?
    if num_packet_types > 1 {
        stream.serialise_int(&mut packet_type, 0, num_packet_types as i32);
    }

    // Serialize the packet
    if !packet.serialize_internal_w(&mut stream) {
        return 0;
    }

    stream.serialize_check(&mut String::from("end of packet"));

    stream.writer.flush();
    let bytes_processed = stream.get_bytes_processed();

    if ProtocolError::None != stream.get_error() {
        return 0;
    }

    // Write crc32 into packet
    if !info.raw_format {
        unsafe {
            // Pointer to buffer start + prefix bytes
            let dest_ptr = (buffer.as_mut_ptr() as *mut u8).add(info.prefix_bytes as usize);
            let protocol_bytes_temp = info.protocol_id.to_le_bytes();
            let protocol_bytes = protocol_bytes_temp.as_slice();
            let buffer_bytes =
                from_raw_parts::<u8>(dest_ptr, buffer_length - info.prefix_bytes as usize);

            let mut crc_bytes: Vec<u8> = vec![];
            crc_bytes.extend_from_slice(protocol_bytes);
            crc_bytes.extend_from_slice(buffer_bytes);

            crc_32 = crc32fast::hash(&crc_bytes);
            let src_ptr = crc_32.to_le_bytes().as_ptr();

            // Write crc32 directly into buffer
            std::ptr::copy_nonoverlapping(src_ptr, dest_ptr, 4);
        }
    }

    return bytes_processed;
}

pub fn read_packet(
    info: &PacketInfo,
    buffer: &mut Buffer,
    header: Option<&mut dyn Object>,
    error: &mut ProtocolError,
) -> Option<Box<dyn Packet>> {
    assert!(buffer.len() > 0);

    if *error != ProtocolError::None {
        *error = ProtocolError::None;
    }

    let buffer_ptr = buffer.as_ptr() as *mut u8;
    let buffer_length = buffer.len();
    let mut stream = ReadStream::new(buffer, buffer.len());
    // stream.SetContext(info.context);

    for _i in 0..info.prefix_bytes {
        let mut dummy: u32 = 0;
        stream.serialize_bits(&mut dummy, 8);
    }

    // TODO: Move crc32 stuff into functions.
    let mut read_crc32 = 0;
    if !info.raw_format {
        stream.serialize_bits(&mut read_crc32, 32);

        // On read, we skip prefix bytes
        // Overwrite CRC with 0's
        // Read the rest.

        unsafe {
            let src_ptr = buffer_ptr.add(info.prefix_bytes as usize + 4);
            let protocol_bytes_temp = info.protocol_id.to_le_bytes();
            let protocol_bytes = protocol_bytes_temp.as_slice();
            let buffer_bytes =
                from_raw_parts::<u8>(src_ptr, buffer_length - info.prefix_bytes as usize - 4);
            let mut crc_bytes: Vec<u8> = vec![];
            crc_bytes.extend_from_slice(protocol_bytes);
            crc_bytes.extend_from_slice(&[0, 0, 0, 0]); // Fill in space that CRC32 was in
            crc_bytes.extend_from_slice(buffer_bytes);

            let crc_32 = crc32fast::hash(&crc_bytes);

            assert_eq!(
                read_crc32, crc_32,
                "Corrupt packet. Expected CRC32: {:?}, got CRC32: {:?}",
                read_crc32, crc_32
            );
        }
    }

    match header {
        Some(header) => {
            if !header.serialize_internal_r(&mut stream) {
                if *error != ProtocolError::None {
                    *error = ProtocolError::SerializeHeaderFailed
                }
            }
        }
        None => (),
    }

    let mut packet_type: u32 = 0;
    let num_packet_types = info.packet_factory.get_num_packet_types();
    assert!(num_packet_types > 0);

    if num_packet_types > 1 {
        let mut temp_packet_type: i32 = 0;
        if !stream.serialise_int(&mut temp_packet_type, 0, num_packet_types as i32) {
            if *error != ProtocolError::None {
                *error = ProtocolError::InvalidPacketType;
                return None;
            }
        }
        packet_type = temp_packet_type as u32;
    }

    if !info.allowed_packet_types.contains(&packet_type) {
        if *error == ProtocolError::None {
            *error = ProtocolError::PacketTypeNotAllowed;
        }
        return None;
    }

    let mut packet = info.packet_factory.create_packet(packet_type);

    if !packet.serialize_internal_r(&mut stream) {
        if *error == ProtocolError::None {
            *error = ProtocolError::SerializePacketFailed;
        }
        return None;
    }

    if !stream.serialize_check(&mut String::from("end of packet")) {
        if *error == ProtocolError::None {
            *error = ProtocolError::SerializeCheckFailed;
        }
        return None;
    }

    if stream.get_error() != ProtocolError::None {
        if *error == ProtocolError::None {
            *error = stream.get_error();
        }
        return None;
    }

    if *error != ProtocolError::None {
        return None;
    }

    return Some(packet);
}
