use super::object::Packet;

/** TODO */
pub trait PacketFactory {
    fn get_num_packet_types(&self) -> u32;
    fn get_num_allocated_packets(&self) -> u32;
    fn create_packet(&self, packet_type: u32) -> Box<dyn Packet>;
    fn destroy_packet(&self);
}
