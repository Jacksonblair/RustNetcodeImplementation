use crate::{
    impl_object_for_packet, packet_factory_methods,
    protocol::{
        packets::{write_packet, Object, PacketFactory, PacketInfo},
        serialization::{serialize_bits_macro, serialize_float_macro, serialize_int_macro},
        streams::{ReadStream, Stream, WriteStream},
    },
};

const NUM_ITERATIONS: u32 = 10;
const PACKET_BUFFER_SIZE: usize = 256;
const MAX_FRAGMENT_SIZE: usize = 1024;
const MAX_FRAGMENTS_PER_PACKET: usize = 256;
const MAX_PACKET_SIZE: usize = MAX_FRAGMENT_SIZE * MAX_FRAGMENTS_PER_PACKET;
const PROTOCOL_ID: u32 = 0x55667788;

/*
   LAST TIME:
   - Realized packet_size variable lives inside each example. duh.

*/

/*  */

enum TestPacketTypes {
    FRAGMENT = 0,
    A = 1,
    B = 2,
    C = 3,
    NumTypes = 4,
}

struct PacketData {
    size: u32,
    data: Vec<u8>,
}

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
        } else if packet_type == TestPacketTypes::B as u32 {
        } else if packet_type == TestPacketTypes::C as u32 {
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

// impl Object for TestPacketHeader {
//     fn serialize_internal_r(&mut self, stream: &mut crate::protocol::streams::ReadStream) -> bool {}
//     fn serialize_internal_w(&mut self, stream: &mut crate::protocol::streams::WriteStream) -> bool {
//     }
// }

pub fn test() {
    let packet_factory = TestPacketFactory::new();
    let sequence: u16 = 0;

    for _ in 0..NUM_ITERATIONS {
        let packet_type: u32 = 1 + rand::random::<u32>() % (TestPacketTypes::NumTypes as u32) - 1; // 0 Indicates packet fragment
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
            let num_fragments: u32 = 0;
            let fragment_packet: Vec<PacketData> = vec![];
        } else {
            println!("Sending packet {:?} as a regular packet", sequence);
        }
    }

    /*

       for ( int i = 0; ( i < NumIterations || NumIterations == -1 ); ++i )
       {
           if ( bytesWritten > MaxFragmentSize )
           {
               int numFragments;
               PacketData fragmentPackets[MaxFragmentsPerPacket];
               SplitPacketIntoFragments( sequence, buffer, bytesWritten, numFragments, fragmentPackets );

               for ( int j = 0; j < numFragments; ++j )
                   packetBuffer.ProcessPacket( fragmentPackets[j].data, fragmentPackets[j].size );
           }
           else
           {
               printf( "sending packet %d as a regular packet\n", sequence );

               packetBuffer.ProcessPacket( buffer, bytesWritten );
           }

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
    buffer: &Vec<u32>,
    packet_size: u32,
    bytes_written: u32,
    num_fragments: &mut u32,
    fragment_packets: Vec<PacketData>,
) -> bool {
    *num_fragments = 0;

    assert!(packet_size > 0);
    assert!(packet_size < MAX_PACKET_SIZE as u32);

    // WHATS THIS FOR THEN AYE
    if (packet_size / MAX_FRAGMENT_SIZE as u32) + (packet_size % MAX_FRAGMENT_SIZE as u32) != 0 {
        *num_fragments = 1
    } else {
        *num_fragments = 0
    };

    // START HERE TOMORROW

    true
}

/*
bool SplitPacketIntoFragments( uint16_t sequence, const uint8_t *packetData, int packetSize, int & numFragments, PacketData fragmentPackets[] )
{
    numFragments = 0;

    assert( packetData );
    assert( packetSize > 0 );
    assert( packetSize < MaxPacketSize );

    numFragments = ( packetSize / MaxFragmentSize ) + ( ( packetSize % MaxFragmentSize ) != 0 ? 1 : 0 );

    assert( numFragments > 0 );
    assert( numFragments <= MaxFragmentsPerPacket );

    const uint8_t *src = packetData;

    printf( "splitting packet into %d fragments\n", numFragments );

    for ( int i = 0; i < numFragments; ++i )
    {
        const int fragmentSize = ( i == numFragments - 1 ) ? ( (int) ( intptr_t( packetData + packetSize ) - intptr_t( src ) ) ) : MaxFragmentSize;

        static const int MaxFragmentPacketSize = MaxFragmentSize + PacketFragmentHeaderBytes;

        fragmentPackets[i].data = new uint8_t[MaxFragmentPacketSize];

        protocol2::WriteStream stream( fragmentPackets[i].data, MaxFragmentPacketSize );

        FragmentPacket fragmentPacket;
        fragmentPacket.fragmentSize = fragmentSize;
        fragmentPacket.crc32 = 0;
        fragmentPacket.sequence = sequence;
        fragmentPacket.fragmentId = (uint8_t) i;
        fragmentPacket.numFragments = (uint8_t) numFragments;
        memcpy( fragmentPacket.fragmentData, src, fragmentSize );

        if ( !fragmentPacket.SerializeInternal( stream ) )
        {
            numFragments = 0;
            for ( int j = 0; j < i; ++j )
            {
                delete fragmentPackets[i].data;
                fragmentPackets[i].data = NULL;
                fragmentPackets[i].size = 0;
            }
            return false;
        }

        stream.Flush();

        uint32_t protocolId = protocol2::host_to_network( ProtocolId );
        uint32_t crc32 = protocol2::calculate_crc32( (uint8_t*) &protocolId, 4 );
        crc32 = protocol2::calculate_crc32( fragmentPackets[i].data, stream.GetBytesProcessed(), crc32 );

        *((uint32_t*)fragmentPackets[i].data) = protocol2::host_to_network( crc32 );

        printf( "fragment packet %d: %d bytes\n", i, stream.GetBytesProcessed() );

        fragmentPackets[i].size = stream.GetBytesProcessed();

        src += fragmentSize;
    }

    assert( src == packetData + packetSize );

    return true;
}
 */
