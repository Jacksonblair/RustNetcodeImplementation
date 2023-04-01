use super::{streams::{ReadStream, WriteStream, Stream}};

#[derive(Copy, Clone)]
pub enum PacketTypes {
    TestPacketA
}

pub trait Object {
    fn serialize_internal_r(&mut self, stream: &mut ReadStream) -> bool;
    fn serialize_internal_w(&mut self, stream: &mut WriteStream) -> bool;
}

pub trait Packet: Object {
    fn get_packet_type() -> PacketTypes;
}


pub struct TestObjectA {
    x: i32,
    y: i32,
    z: i32
}

// - Test Packet A -

pub struct TestPacketA {
    object_a: TestObjectA
}

impl Packet for TestPacketA {
    fn get_packet_type() -> PacketTypes {
        PacketTypes::TestPacketA
    }
}

impl Object for TestPacketA {
    fn serialize_internal_r(&mut self, stream: &mut ReadStream) -> bool {
        stream.serialise_int(&mut self.object_a.x, 0, 20);
        stream.serialise_int(&mut self.object_a.y, 0, 20);
        stream.serialise_int(&mut self.object_a.z, 0, 20);
        true
    }
    fn serialize_internal_w(&mut self, stream: &mut WriteStream) -> bool {
        stream.serialise_int(&mut self.object_a.x, 0, 20);
        stream.serialise_int(&mut self.object_a.y, 0, 20);
        stream.serialise_int(&mut self.object_a.z, 0, 20);
        stream.writer.flush();

        true
    }
}

pub fn test_packet() {
    let mut buffer: Vec<u32> = vec![0; 100];

    let mut writestream = WriteStream::new(&mut buffer);
    let mut pack = TestPacketA{
        object_a: TestObjectA{
            x: 10,
            y: 20,
            z: 30
        }
    };

    pack.serialize_internal_w(&mut writestream);

    for i in 0..buffer.len() {
        println!("{:#032b}", buffer[i]);
    }
}