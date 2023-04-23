pub mod bitpacker;
pub mod macros;
pub mod packets;
pub mod serialization;
pub mod streams;

/*
    NOTE: DONT DELETE
    - We use the macro to delcare the 'Object' methods when we can use the packet.Serialize
    - Otherwise we declare them ourselves

    How we choose from above depends on whether or not we need to separate the read and write serialize functions, or whether they just call .Serialize

    We either implement the object methods or write a Serialzie method in the packet struct and then fulfil Object with the macro.

    "Packet" interface does need a Serialize func, we just implement one if we want all of the Object methods to do the same thing.

*/

/*
    NOTE: DONT DELETE
    Recapping...

    Packet trait implements Object
        Object has:
            - serialize_w
            - serialise_r

    Packets are passed to write_packet method
    - Write packet does not care about the packet type, it just creates a write_stream and calls the packet.serialize_w method with the correct args.
*/

/*
    LAST TIME ON BUILDING A NETWORK PROTOCOL:
    - We realized that some of our serializations didnt work, and decided to write test cases for them. We need to finish all the test cases before continuing, so we know when that shit breaks.

*/

mod test_reading_and_writing_packets {

    const MAX_PACKET_SIZE: usize = 1024;

    use std::fmt::Write;

    use super::packets::PacketFactory;
    use super::serialization::{
        read_object_index_macro, serialize_bool_macro, serialize_float_macro, serialize_object,
        serialize_object_index_internal, write_object_index_macro,
    };
    use crate::impl_object_for_packet;
    use crate::protocol::packets::{self, read_packet, Object, Packet, PacketInfo};
    use crate::protocol::serialization::{serialize_int_macro, MAX_OBJECTS};
    use crate::protocol::streams::{ReadStream, Stream, WriteStream};
    use rand::random;

    const NUM_ITERATIONS: u32 = 16;

    #[derive(Debug)]
    pub struct TestObject {
        send: bool,
        a: i32,
    }

    #[derive(Debug)]
    pub struct SceneA {
        objects: Vec<TestObject>,
    }

    enum TestPacketTypes {
        A,
        B,
        NUM_TYPES,
    }

    #[derive(Debug)]
    pub struct TestPacketA {
        a: u32,
        b: u32,
        c: f32,
    }

    impl TestPacketA {
        fn new() -> TestPacketA {
            return TestPacketA {
                a: random::<u32>(),
                b: random::<u32>(),
                c: random::<f32>(),
            };
        }

        fn serialize(&mut self, stream: &mut dyn Stream) -> bool {
            serialize_int_macro(stream, &mut (self.a as i32), i32::MIN, i32::MAX);
            serialize_int_macro(stream, &mut (self.b as i32), i32::MIN, i32::MAX);
            serialize_float_macro(stream, &mut self.c);
            true
        }
    }

    impl Packet for TestPacketA {
        fn get_packet_type(&self) -> u32 {
            TestPacketTypes::A as u32
        }
    }

    impl_object_for_packet!(TestPacketA);

    pub struct TestPacketB {
        scene: SceneA,
    }

    impl TestPacketB {
        fn new() -> TestPacketB {
            TestPacketB {
                scene: SceneA {
                    objects: (0..10)
                        .map(|_| TestObject {
                            send: true,
                            a: random::<i32>(),
                        })
                        .collect(),
                    // objects: vec![],
                },
            }
        }
    }

    impl Packet for TestPacketB {
        fn get_packet_type(&self) -> u32 {
            TestPacketTypes::B as u32
        }
    }

    impl Object for TestPacketB {
        fn serialize_internal_r(&mut self, stream: &mut ReadStream) -> bool {
            read_scene_a(stream, &mut self.scene);
            true
        }
        fn serialize_internal_w(&mut self, stream: &mut WriteStream) -> bool {
            write_scene_a(stream, &mut self.scene);
            true
        }
    }

    fn write_scene_a(stream: &mut WriteStream, scene: &mut SceneA) -> bool {
        let mut previous_index = -1;

        for i in 0..scene.objects.len() {
            if !scene.objects[i as usize].send {
                continue;
            }
            write_object_index_macro(stream, &mut previous_index, i as i32);
            // Write object
            serialize_int_macro(stream, &mut scene.objects[i as usize].a, i32::MIN, i32::MAX);
        }

        // Write sentinel value
        write_object_index_macro(stream, &mut previous_index, MAX_OBJECTS as i32);
        true
    }

    fn read_scene_a(stream: &mut ReadStream, scene: &mut SceneA) -> bool {
        let mut previous_index = -1;

        loop {
            let mut index = 0;
            read_object_index_macro(stream, &mut previous_index, &mut index);
            if index == MAX_OBJECTS as i32 {
                // When we hit 'sentinel' value
                break;
            }
            // Read object
            serialize_int_macro(
                stream,
                &mut scene.objects[index as usize].a,
                i32::MIN,
                i32::MAX,
            );
        }

        true
    }

    #[derive(Default)]
    struct TestPacketFactory {
        num_allocated_packets: u32,
        num_packet_types: u32,
    }

    impl PacketFactory for TestPacketFactory {
        fn get_num_packet_types(&self) -> u32 {
            self.num_packet_types
        }

        fn create_packet(&self, packet_type: u32) -> Box<dyn Packet> {
            if packet_type == TestPacketTypes::A as u32 {
                Box::new(TestPacketA::new())
            } else if packet_type == TestPacketTypes::B as u32 {
                Box::new(TestPacketB::new())
            } else {
                panic!();
            }
        }

        fn destroy_packet(&self) {
            todo!()
        }

        fn get_num_allocated_packets(&self) -> u32 {
            self.num_allocated_packets
        }
    }

    #[test]
    pub fn reading_and_writing_packets() {
        return;
        let mut buffer: Vec<u32> = vec![0; MAX_PACKET_SIZE];

        {
            let mut write_stream = WriteStream::new(&mut buffer, MAX_PACKET_SIZE);
            let mut write_packet = TestPacketB::new();
            write_packet.serialize_internal_w(&mut write_stream);
            write_stream.writer.flush();
        }

        {
            let mut read_stream = ReadStream::new(&mut buffer, MAX_PACKET_SIZE);
            let mut read_packet = TestPacketB::new();
            read_packet.serialize_internal_r(&mut read_stream);
        }

        // println!("Reading and writing packets\n\n");

        // let packet_factory = TestPacketFactory {
        //     num_allocated_packets: 0,
        //     num_packet_types: TestPacketTypes::NUM_TYPES as u32,
        // };

        // for _i in 0..NUM_ITERATIONS {
        //     let packet_type = random::<u32>() % TestPacketTypes::NUM_TYPES as u32;
        //     let mut write_packet = packet_factory.create_packet(packet_type);

        //     assert!(write_packet.get_packet_type() == packet_type);

        //     let mut buffer: Vec<u32> = vec![0; MAX_PACKET_SIZE];
        //     let mut error = false;

        //     let info: PacketInfo = PacketInfo {
        //         raw_format: false,
        //         prefix_bytes: 1,
        //         protocol_id: 12345,
        //         allowed_packet_types: vec![],
        //         packet_factory: &packet_factory,
        //     };

        //     let bytes_written = packets::write_packet(&info, write_packet.as_mut(), &mut buffer);

        //     for i in 0..3 {
        //         println!("Write buffer: {:#034b}", buffer[i]);
        //     }

        //     if bytes_written > 0 {
        //         println!(
        //             "Wrote packet type {} ({} bytes)\n",
        //             write_packet.get_packet_type(),
        //             bytes_written
        //         );
        //     } else {
        //         println!("Write packet error. Didnt write any bytes.");
        //         error = true;
        //     }

        //     // for i in 0..5 {
        //     //     println!("Write buffer: {:#034b}", write_buffer[i]);
        //     // }

        //     let read_packet = read_packet(&info, &mut buffer);

        //     println!(
        //         "Read packet type {} ({} bytes)",
        //         read_packet.get_packet_type(),
        //         bytes_written
        //     );

        //     println!("\n");

        /*
                memset( readBuffer, 0, sizeof( readBuffer ) );
                memcpy( readBuffer, writeBuffer, bytesWritten );

                int readError;

                protocol2::Packet *readPacket = protocol2::ReadPacket( info, readBuffer, bytesWritten, NULL, &readError );

                if ( readPacket )
                {
                    printf( "read packet type %d (%d bytes)\n", readPacket->GetType(), bytesWritten );
                }
                else
                {
                    printf( "read packet error: %s\n", protocol2::GetErrorString( readError ) );

                    error = true;
                }

                packetFactory.DestroyPacket( readPacket );
                packetFactory.DestroyPacket( writePacket );

                if ( error )
                    return 1;
        */
        // }
    }
}
