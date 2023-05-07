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

const MAX_PACKET_SIZE: usize = 256 * 1024;

use rand::random;

use crate::{
    impl_object_for_packet,
    protocol::{
        get_error_string,
        packets::{self, read_packet, Object, Packet, PacketFactory, PacketInfo},
        serialization::{
            read_object_index_macro, serialize_float_macro, serialize_int_macro,
            write_object_index_macro, MAX_OBJECTS,
        },
        streams::{ReadStream, Stream, WriteStream},
        Buffer, ProtocolError,
    },
};

const NUM_ITERATIONS: u32 = 100;

#[derive(Debug, PartialEq)]
pub struct TestObject {
    send: bool,
    a: i32,
}

#[derive(Debug, PartialEq)]
pub struct SceneA {
    objects: Vec<TestObject>,
}

enum TestPacketTypes {
    A = 0,
    B = 1,
    NumTypes = 2,
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

#[derive(PartialEq)]
pub struct TestPacketB {
    scene: SceneA,
}

impl TestPacketB {
    fn new() -> TestPacketB {
        TestPacketB {
            scene: SceneA {
                objects: (0..100)
                    .map(|_| TestObject {
                        send: random::<bool>(),
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
pub fn test() {
    let packet_factory = TestPacketFactory {
        num_allocated_packets: 0,
        num_packet_types: TestPacketTypes::NumTypes as u32,
    };

    for _i in 0..NUM_ITERATIONS {
        let packet_type = random::<u32>() % TestPacketTypes::NumTypes as u32;
        let mut write_packet: Box<dyn Packet> = packet_factory.create_packet(packet_type);

        assert!(write_packet.get_packet_type() == packet_type);

        let mut buffer: Buffer = vec![0; MAX_PACKET_SIZE];
        let mut error: bool = false;

        let info: PacketInfo = PacketInfo {
            raw_format: false,
            prefix_bytes: 4,
            protocol_id: u32::MAX,
            allowed_packet_types: vec![TestPacketTypes::A as u32, TestPacketTypes::B as u32],
            packet_factory: &packet_factory,
        };

        let bytes_written = packets::write_packet(
            &info,
            write_packet.as_mut(),
            &mut buffer,
            MAX_PACKET_SIZE as usize,
            None,
        );

        if bytes_written > 0 {
            println!(
                "Wrote packet type {} ({} bytes)",
                write_packet.get_packet_type(),
                bytes_written
            );
        } else {
            println!("Write packet error. Didnt write any bytes.");
            error = true;
        }

        let mut error: ProtocolError = ProtocolError::None;
        let read_packet = read_packet(&info, &mut buffer, None, &mut error);
        match read_packet {
            Some(packet) => {
                println!(
                    "Read packet type {} ({} bytes)",
                    packet.get_packet_type(),
                    bytes_written
                );
            }
            None => {
                println!("Packet read error {:?}", get_error_string(error));
            }
        }
    }
}
