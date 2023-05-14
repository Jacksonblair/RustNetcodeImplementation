use rand::Rng;

/*
   SENDING:
   - Get packet, split into fragment packets if needed.
   - Send fragment packets one at a time.

   RECEIVING:
   - Get packet, check if fragment packet
   - If fragment packet, join them back together.



*/

use crate::{
    impl_object_for_packet, packet_factory_methods,
    protocol::{
        constants::*,
        helpers::calc_packet_crc32,
        packets::object::Object,
        packets::{
            fragment_packet::FragmentPacket, object::Packet, packet_buffer::PacketBuffer,
            packet_data::PacketData, packet_factory::PacketFactory, packet_info::PacketInfo,
            write_packet,
        },
        serialization::*,
        streams::{read_stream::ReadStream, write_stream::WriteStream, Stream},
    },
};

const NUM_ITERATIONS: u32 = 10;

/**
Takes a buffer larger than a single packets max since, and:
- Splits into fragment size chunks
- Serializes each fragment (adds data about the fragmentation to each packet)
- Copies that serialized data back into the array passed into the function
*/
fn split_packet_into_fragments(
    sequence: u16,
    packet_data: &mut Buffer,
    big_packet_size: u32,
    num_fragments: &mut u32,
    fragment_packets: &mut Vec<PacketData>,
) -> bool {
    *num_fragments = 0;

    assert!(big_packet_size > 0);
    assert!(big_packet_size < MAX_PACKET_SIZE as u32);

    // Calculate number of fragments in packet
    if big_packet_size % MAX_FRAGMENT_SIZE as u32 != 0 {
        *num_fragments = (big_packet_size / MAX_FRAGMENT_SIZE as u32) + 1
    } else {
        *num_fragments = big_packet_size / MAX_FRAGMENT_SIZE as u32
    };

    assert!(*num_fragments > 0);
    assert!(*num_fragments <= MAX_FRAGMENTS_PER_PACKET as u32);
    println!("Splitting packet into {:?} fragments", *num_fragments);

    // Pointer to our big packets buffer
    let mut src_ptr = packet_data.as_ptr() as *mut u8;

    for i in 0..*num_fragments as usize {
        let mut bytes_processed: u32 = 0;

        // Calculate fragment size
        let fragment_size = if i as u32 == *num_fragments - 1 {
            big_packet_size % (MAX_FRAGMENT_SIZE as u32)
        } else {
            MAX_FRAGMENT_SIZE as u32
        };

        // const MAX_PACKET_FRAGMENT_SIZE is max packet size + number of header bytes
        fragment_packets[i as usize].data = vec![0; MAX_PACKET_FRAGMENT_SIZE];
        let mut stream = WriteStream::new(
            &mut fragment_packets[i as usize].data,
            MAX_PACKET_FRAGMENT_SIZE,
        );

        let mut fragment_packet = FragmentPacket::new();
        fragment_packet.fragment_size = fragment_size;
        fragment_packet.sequence = sequence;
        fragment_packet.fragment_id = i as u8;
        fragment_packet.num_fragments = (*num_fragments) as u8;

        {
            unsafe {
                // Copy fragment_size * bytes into fragment packet data from packet data
                let dest_ptr = fragment_packet.fragment_data.as_mut_ptr();
                std::ptr::copy_nonoverlapping(src_ptr, dest_ptr, fragment_size as usize);
            }

            // Serialize fragment packet into our packet data buffer.
            if !fragment_packet.serialize(&mut stream) {
                *num_fragments = 0;
                for _ in 0..i {
                    fragment_packets[i as usize].size = 0;
                }
                return false;
            };

            stream.writer.flush();
            bytes_processed = stream.get_bytes_processed();
        }

        // -- Calc crc32 and add into packet data
        // TODO: host_to_network(protocolID) ??
        let fragment_packets_data_ptr = &fragment_packets[i as usize].data;
        let crc32 = calc_packet_crc32(fragment_packets_data_ptr, PROTOCOL_ID);
        unsafe {
            std::ptr::copy_nonoverlapping(
                crc32.to_le_bytes().as_ptr(),
                fragment_packets[i].data.as_mut_ptr(),
                4,
            );
        }

        println!("Fragment packet {:?}: {:?} bytes", i, bytes_processed);
        fragment_packets[i as usize].size = bytes_processed;

        unsafe {
            // Advance src ptr by fragment size
            src_ptr = src_ptr.add(fragment_size as usize);
        }
    }

    return true;
}

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
    fn create_packet(&self, packet_type: u32) -> Box<dyn Packet> {
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

#[test]
pub fn test() {
    let mut packet_buffer = PacketBuffer::new();
    let packet_factory = TestPacketFactory::new();
    let sequence: u16 = 0;

    for _ in 0..NUM_ITERATIONS {
        let packet_type: u32 = 1;
        // (1 + rand::random::<u32>()) % ((TestPacketTypes::NumTypes as u32) - 1); // 0 Indicates packet fragment
        println!("PACKET TYPE: {:?}", packet_type);
        let mut packet = packet_factory.create_packet(packet_type);
        assert!(packet.get_packet_type() == packet_type);

        let mut buffer: Buffer = vec![0; MAX_PACKET_SIZE as usize];
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

            // Create vector of PacketData objects to hold data we will split out of our large packet
            let mut fragment_packets: Vec<PacketData> = vec![];
            for _i in 0..MAX_FRAGMENTS_PER_PACKET {
                fragment_packets.push(PacketData::new());
            }

            // Split large packet
            split_packet_into_fragments(
                sequence,
                &mut buffer,
                bytes_written,
                &mut num_fragments,
                &mut fragment_packets,
            );

            // ... sending across the network ...

            // Process the fragment packets
            for j in 0..num_fragments as usize {
                let fragment_size = fragment_packets[j].size;
                packet_buffer.process_packet(&mut fragment_packets[j].data, fragment_size);
            }
        } else {
            println!("Sending packet {:?} as a regular packet", sequence);
            // ... sending across the network ...
            // Process the fragment packet
            packet_buffer.process_packet(&mut buffer, bytes_written);
        }
    }

    //     /*

    //        for ( int i = 0; ( i < NumIterations || NumIterations == -1 ); ++i )
    //        {
    //         ....

    //            int numPackets = 0;
    //            PacketData packets[PacketBufferSize];
    //            packetBuffer.ReceivePackets( numPackets, packets );

    //            for ( int j = 0; j < numPackets; ++j )
    //            {
    //                int readError;
    //                TestPacketHeader readPacketHeader;
    //                protocol2::Packet *readPacket = protocol2::ReadPacket( info, buffer, bytesWritten, &readPacketHeader, &readError );

    //                if ( readPacket )
    //                {
    //                    printf( "read packet type %d (%d bytes)\n", readPacket->GetType(), bytesWritten );

    //                    if ( !CheckPacketsAreIdentical( readPacket, writePacket, readPacketHeader, writePacketHeader ) )
    //                    {
    //                        printf( "failure: read packet is not the same as written packet. something wrong with serialize function?\n" );
    //                        error = true;
    //                    }
    //                    else
    //                    {
    //                        printf( "success: read packet %d matches written packet %d\n", readPacketHeader.sequence, writePacketHeader.sequence );
    //                    }
    //                }
    //                else
    //                {
    //                    printf( "read packet error: %s\n", protocol2::GetErrorString( readError ) );

    //                    error = true;
    //                }

    //                packetFactory.DestroyPacket( readPacket );

    //                if ( error )
    //                    break;

    //                printf( "===================================================\n" );
    //            }

    //            packetFactory.DestroyPacket( writePacket );

    //            if ( error )
    //                return 1;

    //            sequence++;

    //            printf( "\n" );
    //        }

    //        return 0;
    //     */
    // }

    // assert( src == packetData + packetSize );
    // true
}
