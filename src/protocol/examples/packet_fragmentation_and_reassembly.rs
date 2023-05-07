use core::num;

use rand::Rng;

use crate::{
    impl_object_for_packet, packet_factory_methods,
    protocol::{
        calc_packet_crc32,
        packets::{write_packet, Object, Packet, PacketFactory, PacketInfo},
        serialization::{
            serialize_align_macro, serialize_bits_macro, serialize_bytes_internal,
            serialize_float_macro, serialize_int_macro,
        },
        streams::{ReadStream, Stream, WriteStream},
        Buffer,
    },
};

const PROTOCOL_ID: u32 = 0x55667788;
const NUM_ITERATIONS: u32 = 10;

const MAX_FRAGMENT_SIZE: usize = 1024;
const MAX_FRAGMENTS_PER_PACKET: usize = 256;
const MAX_PACKET_SIZE: usize = MAX_FRAGMENT_SIZE * MAX_FRAGMENTS_PER_PACKET;
const PACKET_FRAGMENT_HEADER_BYTES: u32 = 16;
const MAX_PACKET_FRAGMENT_SIZE: u32 = MAX_FRAGMENT_SIZE as u32 + PACKET_FRAGMENT_HEADER_BYTES;

struct PacketBufferEntry<'a> {
    sequence: u32,                  // packet sequence number
    num_fragments: u32,             // number of fragments for this packet
    received_fragments: u32,        // number of received fragments so far
    fragment_size: Vec<u32>,        // size of fragment n in bytes
    fragment_data: &'a mut Vec<u8>, // pointer to data for fragment n
}

struct PacketBuffer<'a> {
    current_sequence: u16,       // sequence number of most recent packet in buffer
    num_buffered_fragments: u32, // total number of fragments stored in the packet buffer (across *all* packets)
    valid: Vec<bool>,            // true if there is a valid buffered packet entry at this index
    entries: Vec<PacketBufferEntry<'a>>, // buffered packets in range [ current_sequence - PacketBufferSize + 1, current_sequence ] (modulo 65536)
}

impl PacketBuffer<'_> {
    fn advance(&mut self) {}
    fn process_fragment(&mut self) {}

    /** TODO */
    fn process_packet(&mut self, data: &mut Buffer, size: usize) -> bool {
        let mut stream = ReadStream::new(data, size);
        let mut fragment_packet = FragmentPacket::new();

        if !fragment_packet.serialize(&mut stream) {
            println!("Error: Fragment packet failed to serialize");
            return false;
        }

        // let crc32 = calc_packet_crc32(data. , PROTOCOL_ID);

        true
    }

    /*
    bool ProcessPacket( const uint8_t *data, int size )
    {
        uint32_t protocolId = protocol2::host_to_network( ProtocolId );
        uint32_t crc32 = protocol2::calculate_crc32( (const uint8_t*) &protocolId, 4 );
        uint32_t zero = 0;
        crc32 = protocol2::calculate_crc32( (const uint8_t*) &zero, 4, crc32 );
        crc32 = protocol2::calculate_crc32( data + 4, size - 4, crc32 );

        if ( crc32 != fragmentPacket.crc32 )
        {
            printf( "corrupt packet: expected crc32 %x, got %x\n", crc32, fragmentPacket.crc32 );
            return false;
        }

        if ( fragmentPacket.packetType == 0 )
        {
            return ProcessFragment( data + PacketFragmentHeaderBytes, fragmentPacket.fragmentSize, fragmentPacket.sequence, fragmentPacket.fragmentId, fragmentPacket.numFragments );
        }
        else
        {
            return ProcessFragment( data, size, fragmentPacket.sequence, 0, 1 );
        }

        return true;
    }
     */
}

/*
    PacketBuffer() { memset( this, 0, sizeof( PacketBuffer ) ); }

uint16_t currentSequence;                           // sequence number of most recent packet in buffer

int numBufferedFragments;                           // total number of fragments stored in the packet buffer (across *all* packets)

bool valid[PacketBufferSize];                       // true if there is a valid buffered packet entry at this index

PacketBufferEntry entries[PacketBufferSize];        // buffered packets in range [ current_sequence - PacketBufferSize + 1, current_sequence ] (modulo 65536)

/*
    Advance the current sequence for the packet buffer forward.
    This function removes old packet entries and frees their fragments.
*/

void Advance( uint16_t sequence )
{
    if ( !protocol2::sequence_greater_than( sequence, currentSequence ) )
        return;

    const uint16_t oldestSequence = sequence - PacketBufferSize + 1;

    for ( int i = 0; i < PacketBufferSize; ++i )
    {
        if ( valid[i] )
        {
            if ( protocol2::sequence_less_than( entries[i].sequence, oldestSequence ) )
            {
                printf( "remove old packet entry %d\n", entries[i].sequence );

                for ( int j = 0; j < (int) entries[i].numFragments; ++j )
                {
                    if ( entries[i].fragmentData[j] )
                    {
                        delete [] entries[i].fragmentData[j];
                        assert( numBufferedFragments > 0 );
                        numBufferedFragments--;
                    }
                }
            }

            memset( &entries[i], 0, sizeof( PacketBufferEntry ) );

            valid[i] = false;
        }
    }

    currentSequence = sequence;
}

/*
    Process packet fragment on receiver side.

    Stores each fragment ready to receive the whole packet once all fragments for that packet are received.

    If any fragment is dropped, fragments are not resent, the whole packet is dropped.

    NOTE: This function is fairly complicated because it must handle all possible cases
    of maliciously constructed packets attempting to overflow and corrupt the packet buffer!
*/

bool ProcessFragment( const uint8_t *fragmentData, int fragmentSize, uint16_t packetSequence, int fragmentId, int numFragmentsInPacket )
{
    assert( fragmentData );

    // fragment size is <= zero? discard the fragment.

    if ( fragmentSize <= 0 )
        return false;

    // fragment size exceeds max fragment size? discard the fragment.

    if ( fragmentSize > MaxFragmentSize )
        return false;

    // num fragments outside of range? discard the fragment

    if ( numFragmentsInPacket <= 0 || numFragmentsInPacket > MaxFragmentsPerPacket )
        return false;

    // fragment index out of range? discard the fragment

    if ( fragmentId < 0 || fragmentId >= numFragmentsInPacket )
        return false;

    // if this is not the last fragment in the packet and fragment size is not equal to MaxFragmentSize, discard the fragment

    if ( fragmentId != numFragmentsInPacket - 1 && fragmentSize != MaxFragmentSize )
        return false;

    // packet sequence number wildly out of range from the current sequence? discard the fragment

    if ( protocol2::sequence_difference( packetSequence, currentSequence ) > 1024 )
        return false;

    // if the entry exists, but has a different sequence number, discard the fragment

    const int index = packetSequence % PacketBufferSize;

    if ( valid[index] && entries[index].sequence != packetSequence )
        return false;

    // if the entry does not exist, add an entry for this sequence # and set total fragments

    if ( !valid[index] )
    {
        Advance( packetSequence );
        entries[index].sequence = packetSequence;
        entries[index].numFragments = numFragmentsInPacket;
        assert( entries[index].receivedFragments == 0 );            // IMPORTANT: Should have already been cleared to zeros in "Advance"
        valid[index] = true;
    }

    // at this point the entry must exist and have the same sequence number as the fragment

    assert( valid[index] );
    assert( entries[index].sequence == packetSequence );

    // if the total number fragments is different for this packet vs. the entry, discard the fragment

    if ( numFragmentsInPacket != (int) entries[index].numFragments )
        return false;

    // if this fragment has already been received, ignore it because it must have come from a duplicate packet

    assert( fragmentId < numFragmentsInPacket );
    assert( fragmentId < MaxFragmentsPerPacket );
    assert( numFragmentsInPacket <= MaxFragmentsPerPacket );

    if ( entries[index].fragmentSize[fragmentId] )
        return false;

    // add the fragment to the packet buffer

    printf( "added fragment %d of packet %d to buffer\n", fragmentId, packetSequence );

    assert( fragmentSize > 0 );
    assert( fragmentSize <= MaxFragmentSize );

    entries[index].fragmentSize[fragmentId] = fragmentSize;
    entries[index].fragmentData[fragmentId] = new uint8_t[fragmentSize];
    memcpy( entries[index].fragmentData[fragmentId], fragmentData, fragmentSize );
    entries[index].receivedFragments++;

    assert( entries[index].receivedFragments <= entries[index].numFragments );

    numBufferedFragments++;

    return true;
}

bool ProcessPacket( const uint8_t *data, int size )
{
    protocol2::ReadStream stream( data, size );

    FragmentPacket fragmentPacket;

    if ( !fragmentPacket.SerializeInternal( stream ) )
    {
        printf( "error: fragment packet failed to serialize\n" );
        return false;
    }

    uint32_t protocolId = protocol2::host_to_network( ProtocolId );
    uint32_t crc32 = protocol2::calculate_crc32( (const uint8_t*) &protocolId, 4 );
    uint32_t zero = 0;
    crc32 = protocol2::calculate_crc32( (const uint8_t*) &zero, 4, crc32 );
    crc32 = protocol2::calculate_crc32( data + 4, size - 4, crc32 );

    if ( crc32 != fragmentPacket.crc32 )
    {
        printf( "corrupt packet: expected crc32 %x, got %x\n", crc32, fragmentPacket.crc32 );
        return false;
    }

    if ( fragmentPacket.packetType == 0 )
    {
        return ProcessFragment( data + PacketFragmentHeaderBytes, fragmentPacket.fragmentSize, fragmentPacket.sequence, fragmentPacket.fragmentId, fragmentPacket.numFragments );
    }
    else
    {
        return ProcessFragment( data, size, fragmentPacket.sequence, 0, 1 );
    }

    return true;
}

void ReceivePackets( int & numPackets, PacketData packetData[] )
{
    numPackets = 0;

    const uint16_t oldestSequence = currentSequence - PacketBufferSize + 1;

    for ( int i = 0; i < PacketBufferSize; ++i )
    {
        const uint16_t sequence = uint16_t( ( oldestSequence + i ) & 0xFF );

        const int index = sequence % PacketBufferSize;

        if ( valid[index] && entries[index].sequence == sequence )
        {
            // have all fragments arrived for this packet?

            if ( entries[index].receivedFragments != entries[index].numFragments )
                continue;

            printf( "received all fragments for packet %d [%d]\n", sequence, entries[index].numFragments );

            // what's the total size of this packet?

            int packetSize = 0;
            for ( int j = 0; j < (int) entries[index].numFragments; ++j )
            {
                packetSize += entries[index].fragmentSize[j];
            }

            assert( packetSize > 0 );
            assert( packetSize <= MaxPacketSize );

            // allocate a packet to return to the caller

            PacketData & packet = packetData[numPackets++];

            packet.size = packetSize;
            packet.data = new uint8_t[packetSize];

            // reconstruct the packet from the fragments

            printf( "reassembling packet %d from fragments (%d bytes)\n", sequence, packetSize );

            uint8_t *dst = packet.data;
            for ( int j = 0; j < (int) entries[index].numFragments; ++j )
            {
                memcpy( dst, entries[index].fragmentData[i], entries[index].fragmentSize[i] );
                dst += entries[index].fragmentSize[i];
            }

            // free all fragments

            for ( int j = 0; j < (int) entries[index].numFragments; ++j )
            {
                delete [] entries[index].fragmentData[j];
                numBufferedFragments--;
            }

            // clear the packet buffer entry

            memset( &entries[index], 0, sizeof( PacketBufferEntry ) );

            valid[index] = false;
        }
    }
}

 */

// fragment packet on-the-wire format:
// [crc32] (32 bits) | [sequence] (16 bits) | [packet type 0] (# of bits depends on number of packet types)
// [fragment id] (8 bits) | [num fragments] (8 bits) | (pad zero bits to nearest byte) | <fragment data>
struct FragmentPacket {
    fragment_size: u32, // set as input on serialize write. output on serialize read (inferred from size of packet)

    crc32: u32,
    sequence: u16,
    packet_type: u32,
    fragment_id: u8,
    num_fragments: u8,
    fragment_data: Vec<u8>,
}

impl FragmentPacket {
    fn new() -> FragmentPacket {
        return FragmentPacket {
            fragment_size: 0,
            crc32: 0,
            sequence: 0,
            packet_type: 0,
            fragment_id: 0,
            num_fragments: 0,
            fragment_data: vec![0; MAX_FRAGMENT_SIZE],
        };
    }

    fn serialize(&mut self, stream: &mut dyn Stream) -> bool {
        serialize_bits_macro(stream, &mut self.crc32, 32);
        serialize_bits_macro(stream, &mut (self.sequence as u32), 16);
        self.packet_type = 0;
        serialize_int_macro(
            stream,
            &mut (self.packet_type as i32),
            0,
            TestPacketTypes::NumTypes as i32 - 1,
        );

        // If packet type is fragment, then return ???
        if self.packet_type == 0 {
            true;
        }

        serialize_bits_macro(stream, &mut (self.fragment_id as u32), 8);
        serialize_bits_macro(stream, &mut (self.num_fragments as u32), 8);

        // Align to next byte ??
        serialize_align_macro(stream);

        if stream.is_reading() {
            // assert!((stream.get_bits_remaining() % 8) == 0)
            // self.fragment_size = stream.get_bits_remaining() / 8;
            if self.fragment_size <= 0 || self.fragment_size > MAX_FRAGMENT_SIZE as u32 {
                println!(
                    "Packet fragment size is out of bounds ({:?})",
                    self.fragment_size
                );
                return false;
            }
        }

        assert!(self.fragment_size > 0);
        assert!(self.fragment_size <= MAX_FRAGMENT_SIZE as u32);

        serialize_bytes_internal(stream, &mut self.fragment_data, self.fragment_size);

        /*
        template <typename Stream> bool Serialize( Stream & stream )
         {
             serialize_bits( stream, crc32, 32 );
             serialize_bits( stream, sequence, 16 );

             packetType = 0;
             serialize_int( stream, packetType, 0, TEST_PACKET_NUM_TYPES - 1 );
             if ( packetType != 0 )
                 return true;

             serialize_bits( stream, fragmentId, 8 );
             serialize_bits( stream, numFragments, 8 );

             serialize_align( stream );

             if ( Stream::IsReading )
             {
                 assert( ( stream.GetBitsRemaining() % 8 ) == 0 );
                 fragmentSize = stream.GetBitsRemaining() / 8;
                 if ( fragmentSize <= 0 || fragmentSize > MaxFragmentSize )
                 {
                     printf( "packet fragment size is out of bounds (%d)\n", fragmentSize );
                     return false;
                 }
             }

             assert( fragmentSize > 0 );
             assert( fragmentSize <= MaxFragmentSize );

             serialize_bytes( stream, fragmentData, fragmentSize );

             return true;
         }
         */

        true
    }
}

struct PacketData {
    size: u32,
    data: Vec<u8>,
}

impl PacketData {
    fn new() -> PacketData {
        PacketData {
            size: 0,
            data: vec![0; MAX_PACKET_FRAGMENT_SIZE as usize],
        }
    }
}

impl_object_for_packet!(FragmentPacket);
enum TestPacketTypes {
    FRAGMENT = 0,
    A = 1,
    // B = 2,
    // C = 3,
    NumTypes = 2,
}

const MAX_TEST_PACKET_A_ITEMS: usize = 4096 * 4;

struct TestPacketA {
    items: Vec<i32>,
}

impl Packet for TestPacketA {
    fn get_packet_type(&self) -> u32 {
        TestPacketTypes::A as u32
    }
}

impl TestPacketA {
    fn new() -> TestPacketA {
        let mut pack = TestPacketA {
            items: vec![0; MAX_TEST_PACKET_A_ITEMS],
        };
        let mut rng = rand::thread_rng();
        for i in 0..pack.items.len() {
            pack.items[i] = rng.gen_range(-100..100);
        }
        return pack;
    }

    fn serialize(&mut self, stream: &mut dyn Stream) -> bool {
        for i in 0..self.items.len() {
            serialize_int_macro(stream, &mut self.items[i], -100, 100);
        }
        true
    }
}

impl_object_for_packet!(TestPacketA);

struct TestPacketFactory {
    num_packet_types: u32,
    num_allocated_packets: u32,
}

impl TestPacketFactory {
    pub fn new() -> TestPacketFactory {
        return TestPacketFactory {
            num_packet_types: TestPacketTypes::NumTypes as u32,
            num_allocated_packets: 0,
        };
    }
}

impl PacketFactory for TestPacketFactory {
    fn create_packet(&self, packet_type: u32) -> Box<dyn crate::protocol::packets::Packet> {
        if packet_type == TestPacketTypes::A as u32 {
            return Box::new(TestPacketA::new());
        }
        panic!();
    }

    packet_factory_methods!();
}

struct TestPacketHeader {
    sequence: u16,
}

impl TestPacketHeader {
    fn serialize(&mut self, stream: &mut dyn Stream) -> bool {
        let mut temp_sequence: u32 = self.sequence as u32;
        serialize_bits_macro(stream, &mut temp_sequence, 16);
        if stream.is_reading() {
            self.sequence = temp_sequence as u16;
        }
        true
    }
}

impl_object_for_packet!(TestPacketHeader);

// #[test]
pub fn test() {
    let packet_factory = TestPacketFactory::new();
    let sequence: u16 = 0;

    for _ in 0..NUM_ITERATIONS {
        let packet_type: u32 = 1;
        // (1 + rand::random::<u32>()) % ((TestPacketTypes::NumTypes as u32) - 1); // 0 Indicates packet fragment
        println!("PACKET TYPE: {:?}", packet_type);

        let mut packet = packet_factory.create_packet(packet_type);

        assert!(packet.get_packet_type() == packet_type);

        let mut buffer: Vec<u32> = vec![0; MAX_PACKET_SIZE as usize];
        let mut error: bool = false;

        let mut write_packet_header = TestPacketHeader { sequence };

        let mut info = PacketInfo::new(&packet_factory);
        info.protocol_id = PROTOCOL_ID;

        let bytes_written = write_packet(
            &info,
            packet.as_mut(),
            &mut buffer,
            MAX_PACKET_SIZE,
            Some(&mut write_packet_header),
        );

        println!("====================================");
        println!("Writing packet {:?}", sequence);

        if bytes_written > 0 {
            println!(
                "Wrote packet type {:?} ({:?} bytes)",
                packet.get_packet_type(),
                bytes_written
            );
        } else {
            println!("Write packet error");
            error = true;
        }

        if bytes_written > MAX_FRAGMENT_SIZE as u32 {
            let mut num_fragments: u32 = 0;
            let mut fragment_packets: Vec<PacketData> = vec![];
            for _i in 0..MAX_FRAGMENTS_PER_PACKET {
                fragment_packets.push(PacketData::new());
            }

            split_packet_into_fragments(
                sequence,
                &mut buffer,
                bytes_written,
                &mut num_fragments,
                &mut fragment_packets,
            );
            // for ( int j = 0; j < num_fragments; ++j )
            //      packetBuffer.process_packet( fragment_packets[j].data, fragment_packets[j].size );
        } else {
            println!("Sending packet {:?} as a  regular packet", sequence);
            // packetBuffer.ProcessPacket( buffer, bytesWritten );
        }
    }

    /*

       for ( int i = 0; ( i < NumIterations || NumIterations == -1 ); ++i )
       {
        ....

           int numPackets = 0;
           PacketData packets[PacketBufferSize];
           packetBuffer.ReceivePackets( numPackets, packets );

           for ( int j = 0; j < numPackets; ++j )
           {
               int readError;
               TestPacketHeader readPacketHeader;
               protocol2::Packet *readPacket = protocol2::ReadPacket( info, buffer, bytesWritten, &readPacketHeader, &readError );

               if ( readPacket )
               {
                   printf( "read packet type %d (%d bytes)\n", readPacket->GetType(), bytesWritten );

                   if ( !CheckPacketsAreIdentical( readPacket, writePacket, readPacketHeader, writePacketHeader ) )
                   {
                       printf( "failure: read packet is not the same as written packet. something wrong with serialize function?\n" );
                       error = true;
                   }
                   else
                   {
                       printf( "success: read packet %d matches written packet %d\n", readPacketHeader.sequence, writePacketHeader.sequence );
                   }
               }
               else
               {
                   printf( "read packet error: %s\n", protocol2::GetErrorString( readError ) );

                   error = true;
               }

               packetFactory.DestroyPacket( readPacket );

               if ( error )
                   break;

               printf( "===================================================\n" );
           }

           packetFactory.DestroyPacket( writePacket );

           if ( error )
               return 1;

           sequence++;

           printf( "\n" );
       }

       return 0;
    */
}

fn split_packet_into_fragments(
    sequence: u16,
    packet_data: &mut Vec<u32>,
    packet_size: u32,
    num_fragments: &mut u32,
    fragment_packets: &mut Vec<PacketData>,
) -> bool {
    *num_fragments = 0;

    assert!(packet_size > 0);
    assert!(packet_size < MAX_PACKET_SIZE as u32);

    // Calculate number of fragments in packet
    if packet_size % MAX_FRAGMENT_SIZE as u32 != 0 {
        *num_fragments = (packet_size / MAX_FRAGMENT_SIZE as u32) + 1
    } else {
        *num_fragments = packet_size / MAX_FRAGMENT_SIZE as u32
    };

    assert!(*num_fragments > 0);
    assert!(*num_fragments <= MAX_FRAGMENTS_PER_PACKET as u32);
    println!("Splitting packet into {:?} fragments", *num_fragments);

    let mut src_ptr = packet_data.as_ptr() as *mut u8;

    for i in 0..*num_fragments {
        // Calculate fragment size
        let fragment_size = if i == *num_fragments - 1 {
            // If at last fragment, get number of remaining bytes/bits?
            packet_size % (MAX_FRAGMENT_SIZE as u32)
        } else {
            MAX_FRAGMENT_SIZE as u32
        };

        assert!(fragment_packets[i as usize].data.len() == MAX_PACKET_FRAGMENT_SIZE as usize);

        let mut stream = WriteStream::new(packet_data, MAX_PACKET_FRAGMENT_SIZE as usize);

        let mut fragment_packet = FragmentPacket::new();
        fragment_packet.fragment_size = fragment_size;
        fragment_packet.sequence = sequence;
        fragment_packet.fragment_id = i as u8;
        fragment_packet.num_fragments = (*num_fragments) as u8;

        // Copy fragment_size * bytes into fragment data from packet data
        unsafe {
            let dest_ptr = fragment_packet.fragment_data.as_mut_ptr();
            std::ptr::copy_nonoverlapping(src_ptr, dest_ptr, fragment_size as usize);
        }

        // Call .serialize on fragment_packet
        if !fragment_packet.serialize(&mut stream) {
            // If serialize fails, do whatever this is for??
            *num_fragments = 0;
            for _ in 0..i {
                // delete fragment_packets[i].data
                fragment_packets[i as usize].size = 0;
                // fragment_packets[i].data = null
            }
            return false;
        };

        stream.writer.flush();
        // TODO: host_to_network(protocolID) ??
        let crc32 = calc_packet_crc32(&fragment_packets[i as usize].data, PROTOCOL_ID);

        // Write crc32 directly into packet data
        unsafe {
            std::ptr::copy_nonoverlapping(
                crc32.to_le_bytes().as_ptr(),
                fragment_packet.fragment_data.as_mut_ptr(),
                4,
            );
        }

        println!(
            "Fragment packet {:?}: {:?} bytes",
            i,
            stream.get_bytes_processed()
        );
        fragment_packets[i as usize].size = stream.get_bytes_processed();

        // Advance src ptr by bytes written
        unsafe {
            src_ptr = src_ptr.add(fragment_size as usize);
        }
    }

    // assert( src == packetData + packetSize );
    true
}
