mod g930x;

extern crate x11;
extern crate pcap;
extern crate libusb;

fn main() {
    g930x::start();
}
