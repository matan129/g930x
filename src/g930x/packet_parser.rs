use pcap::Packet;
use g930x::events::*;

pub struct PacketParser {}

impl PacketParser {
    pub fn new() -> Self {
        Self {}
    }

    pub fn parse(&mut self, packet: &Packet) -> Result<G930KeyEvent, String> {
        let mut leftover = packet.data.into_iter().skip(65);
        let first = leftover.next().unwrap();
        let second = leftover.next().unwrap();
        let marker = ((*first as u16) << 8) | ((*second as u16) & 0xFF);

        match marker {
            0xC => Ok(G930KeyEvent::new(G930Keys::UNKNOWN, G930KeyEventTypes::KeyUp)),
            0x400C => Ok(G930KeyEvent::new(G930Keys::G1, G930KeyEventTypes::KeyDown)),
            0x800C => Ok(G930KeyEvent::new(G930Keys::G2, G930KeyEventTypes::KeyDown)),
            0xD => Ok(G930KeyEvent::new(G930Keys::G3, G930KeyEventTypes::KeyDown)),
            _ => Err(format!("Could not figure out key! marker {}", marker))
        }
    }
}
