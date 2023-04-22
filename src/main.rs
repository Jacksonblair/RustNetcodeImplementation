pub mod protocol;

fn main() {

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
