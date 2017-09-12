use pcap::Packet;

pub trait PacketHandler {
    fn new() -> Self;
    fn handle(&mut self, packet: &Packet) -> Result<(), &'static str>;
}
