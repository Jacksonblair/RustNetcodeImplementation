pub mod protocol;


trait Packet {
    fn Serialise(&mut self, stream: &mut impl protocol::streams::Stream) -> bool;
}

struct PacketA {
    x: u32,
    y: u32,
    z: u32
}

impl PacketA {
    fn new(x: u32, y: u32, z: u32) -> PacketA {
        return PacketA { x, y, z };
    }
}

impl Packet for PacketA {
    fn Serialise(&mut self, stream: &mut impl protocol::streams::Stream) -> bool {

        serialise_int!(stream, self.x, 0 as u32, u32::max_value());
        serialise_int!(stream, self.y, 0 as u32, u32::max_value());
        serialise_int!(stream, self.z, 0 as u32, u32::max_value());

        return true;
    }
}

const max_items: u32 = 32;

struct PacketB {
    num_items: u32,
    items: Vec<u32>
}

impl PacketB {
    fn new(items: &Vec<u32>) -> PacketB {
        return PacketB { num_items: items.len() as u32, items: items.to_vec() };
    }
}

impl Packet for PacketB {
    fn Serialise(&mut self, stream: &mut impl protocol::streams::Stream) -> bool {
        // * Comparison useless because unsigned int, change later.
        serialise_int!(stream, self.num_items, 0, 300000);
        for i in 1..self.items.len() {
            println!("Serialising: {:?}", self.items[i]);
            serialise_int!(stream, self.items[i], 0, 20);
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

fn main() {
    let factory = protocol::packet::PacketFactory::new(1);
    let packet = factory.create_packet(1);

    let mut read_buffer: [u8; MAX_PACKET_SIZE];

    /*
        w bevy:

        


    */



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
