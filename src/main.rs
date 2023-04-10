use protocol::{packets::test_packets::test_packet, serialization::serialize_int_macro};

pub mod protocol;

trait Packet {
    fn Serialise(&mut self, stream: &mut impl protocol::streams::Stream) -> bool;
}

struct PacketA {
    x: i32,
    y: i32,
    z: i32,
}

impl PacketA {
    fn new(x: i32, y: i32, z: i32) -> PacketA {
        return PacketA { x, y, z };
    }
}

impl Packet for PacketA {
    fn Serialise(&mut self, stream: &mut impl protocol::streams::Stream) -> bool {
        serialize_int_macro(stream, &mut self.x, 0, i32::max_value());
        serialize_int_macro(stream, &mut self.y, 0, i32::max_value());
        serialize_int_macro(stream, &mut self.z, 0, i32::max_value());

        return true;
    }
}

const max_items: u32 = 32;

struct PacketB {
    num_items: u32,
    items: Vec<u32>,
}

impl PacketB {
    fn new(items: &Vec<u32>) -> PacketB {
        return PacketB {
            num_items: items.len() as u32,
            items: items.to_vec(),
        };
    }
}

impl Packet for PacketB {
    fn Serialise(&mut self, stream: &mut impl protocol::streams::Stream) -> bool {
        // * Comparison useless because unsigned int, change later.
        serialize_int_macro(stream, &mut (self.num_items as i32), 0, 300000);
        for i in 1..self.items.len() {
            println!("Serialising: {:?}", self.items[i]);
            serialize_int_macro(stream, &mut (self.items[i] as i32), 0, 20);
        }
        return true;
    }
}

/*
    WritePacket
        (packetInfo, Packet, buffer, bufferSize, )

    buffer
    stream
        - reader/writer
            - &buffer

    packet(stream)
        reader/writer(packet)
            -> into buffer


*/

const MAX_PACKET_SIZE: usize = 100;

// fn write_scene_a(stream: &mut dyn Stream, )
/*
    Packet Serialize
        serialize_some_thing(stream, scene)

    Packet SerializeInternal
        write_scene_b(stream, scene)
*/

fn main() {
    test_packet()

    // some_fn();

    // let mut w_stream_1 = streams::WriteStream::new(&mut buffer);

    // let mut packet_a = PacketA::new(u32::max_value(), u32::max_value(), u32::max_value());
    // packet_a.Serialise(&mut w_stream_1);

    // for v in buffer.iter().enumerate() {
    //     println!("{:#034b}", v.1);
    // }

    // let mut r_stream_1 = streams::ReadStream::new(&mut buffer);

    // let mut packet_a2 = PacketA::new(0, 0, 0);

    // packet_a2.Serialise(&mut r_stream_1);

    // println!("{:?}", packet_a2.x);
    // println!("{:?}", packet_a2.y);
    // println!("{:?}", packet_a2.z);

    // let items = vec![1,2,3,4];
    // let mut packet_b = PacketB::new(&items);
    // let mut stream2 = protocol::streams::WriteStream::new(&mut buffer);

    // packet_b.Serialise(&mut stream2);

    // for v in buffer.iter().enumerate() {
    //     println!("{:#034b}", v.1);
    // }

    // let mut packet_a_2 = PacketA{ x: 0, y: 0, z: 0 };

    // // Only want to send 5 bytes, but i can just chuck that into a vec32 on the receiving side.

    // packet_a_2.read(&mut reader);
}
