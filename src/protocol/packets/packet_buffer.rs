use crate::protocol::{constants::*, helpers::calc_packet_crc32, streams::read_stream::ReadStream};

use super::fragment_packet::FragmentPacket;

pub struct PacketBufferEntry<'a> {
    sequence: u16,                  // packet sequence number
    num_fragments: u32,             // number of fragments for this packet
    received_fragments: u32,        // number of received fragments so far
    fragment_size: Vec<u32>,        // size of fragment n in bytes
    fragment_data: &'a mut Vec<u8>, // pointer to data for fragment n
}

/** PacketBuffer is used to process packets as a RECEIVER */
pub struct PacketBuffer<'a> {
    pub current_sequence: u16, // sequence number of most recent packet in buffer
    pub num_buffered_fragments: u32, // total number of fragments stored in the packet buffer (across *all* packets)
    pub valid: [bool; PACKET_BUFFER_SIZE], // true if there is a valid buffered packet entry at this index
    pub entries: Vec<PacketBufferEntry<'a>>, // buffered packets in range [ current_sequence - PacketBufferSize + 1, current_sequence ] (modulo 65536)
}

impl PacketBuffer<'_> {
    pub fn new() -> PacketBuffer<'static> {
        return PacketBuffer {
            current_sequence: 0,
            num_buffered_fragments: 0,
            valid: [false; PACKET_BUFFER_SIZE],
            entries: vec![],
        };
    }

    /**
        Process packet fragment

        - Stores each fragment ready to receive the whole packet once all fragments for that packet are received.
        - If any fragment is dropped, fragments are not resent, the whole packet is dropped.

        NOTE: This function is fairly complicated because it must handle all possible cases
        of maliciously constructed packets attempting to overflow and corrupt the packet buffer!
    */
    pub fn process_fragment(
        &mut self,
        fragment_data: &mut Buffer,
        fragment_size: u32,
        packet_sequence: u16,
        fragment_id: usize,
        num_fragments_in_packet: u32,
    ) -> bool {
        // Fragment size <= 0 ? Discard the fragment
        if fragment_size <= 0 {
            return false;
        }

        // fragment size exceeds max fragment size? discard the fragment.
        if fragment_size as usize > MAX_FRAGMENT_SIZE {
            return false;
        }

        // num fragments outside of range? discard the fragment
        if num_fragments_in_packet <= 0
            || num_fragments_in_packet as usize > MAX_FRAGMENTS_PER_PACKET
        {
            return false;
        }

        // fragment index out of range? discard the fragment
        if fragment_id <= 0 || fragment_id >= num_fragments_in_packet.try_into().unwrap() {
            return false;
        }

        // if this is not the last fragment in the packet and fragment size is not equal to MaxFragmentSize, discard the fragment
        if fragment_id as u32 != num_fragments_in_packet - 1
            && fragment_size != MAX_FRAGMENT_SIZE as u32
        {
            return false;
        }

        // packet sequence number wildly out of range from the current sequence? discard the fragment

        // if ( protocol2::sequence_difference( packetSequence, currentSequence ) > 1024 )
        //     return false;

        // if the entry exists, but has a different sequence number, discard the fragment
        let index = packet_sequence as usize % PACKET_BUFFER_SIZE;
        if self.valid[index] && self.entries[index].sequence != packet_sequence {
            return false;
        }

        // if the entry does not exist, add an entry for this sequence # and set total fragments
        if !self.valid[index] {
            // Advance( packetSequence );
            // entries[index].sequence = packetSequence;
            // entries[index].numFragments = numFragmentsInPacket;
            // assert( entries[index].receivedFragments == 0 );            // IMPORTANT: Should have already been cleared to zeros in "Advance"
            // valid[index] = true;
        }

        // at this point the entry must exist and have the same sequence number as the fragment
        assert!(self.valid[index]);
        assert!(self.entries[index].sequence == packet_sequence);

        // if the total number fragments is different for this packet vs. the entry, discard the fragment
        if num_fragments_in_packet != self.entries[index].num_fragments {
            return false;
        }

        // if this fragment has already been received, ignore it because it must have come from a duplicate packet
        assert!(fragment_id < num_fragments_in_packet.try_into().unwrap());
        assert!(fragment_id < MAX_FRAGMENTS_PER_PACKET.try_into().unwrap());
        assert!(num_fragments_in_packet <= MAX_FRAGMENTS_PER_PACKET as u32);

        if self.entries[index].fragment_size[fragment_id] != 0 {
            return false;
        }

        // add the fragment to the packet buffer
        println!(
            "Added fragment {:?} of packet {:?} to buffer",
            fragment_id, packet_sequence
        );

        assert!(fragment_size > 0);
        assert!(fragment_size <= MAX_FRAGMENT_SIZE.try_into().unwrap());

        self.entries[index].fragment_size[fragment_id] = fragment_size;
        // entries[index].fragmentData[fragmentId] = new uint8_t[fragmentSize];
        // memcpy( entries[index].fragmentData[fragmentId], fragmentData, fragmentSize );
        self.entries[index].received_fragments += 1;

        assert!(self.entries[index].received_fragments <= self.entries[index].num_fragments);

        self.num_buffered_fragments += 1;

        true
    }

    /**
        Advance the current sequence for the packet buffer forward.
        This function removes old packet entries and frees their fragments.
    */
    fn advance(&mut self) {}

    /** Method for processing a RECEIVED packet */
    pub fn process_packet(&mut self, data: &mut Buffer, size: u32) -> bool {
        let mut stream = ReadStream::new(data, size as usize);
        let mut fragment_packet = FragmentPacket::new();

        // Serialize the packet data into the fragment_packet
        if !fragment_packet.serialize(&mut stream) {
            println!("Error: Fragment packet failed to serialize");
            return false;
        }

        /*
            uint32_t protocolId = protocol2::host_to_network( ProtocolId );
            uint32_t crc32 = protocol2::calculate_crc32( (const uint8_t*) &protocolId, 4 );
            uint32_t zero = 0;
            crc32 = protocol2::calculate_crc32( (const uint8_t*) &zero, 4, crc32 );
            crc32 = protocol2::calculate_crc32( data + 4, size - 4, crc32 );
        */

        let crc32 = calc_packet_crc32(data, PROTOCOL_ID);

        if crc32 != fragment_packet.crc32 {
            println!(
                "Corrupt packet: Expected crc32 {:?}, got {:?}",
                crc32, fragment_packet.crc32
            );
        }

        if fragment_packet.packet_type == 0 {
            // return ProcessFragment( data + PacketFragmentHeaderBytes, fragmentPacket.fragmentSize, fragmentPacket.sequence, fragmentPacket.fragmentId, fragmentPacket.numFragments );
        } else {
            // return ProcessFragment( data, size, fragmentPacket.sequence, 0, 1 );
        }

        true
    }
}
