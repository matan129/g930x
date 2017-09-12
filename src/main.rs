mod g930x;

extern crate x11_dl;
extern crate pcap;
extern crate libusb;

fn main() {
    g930x::start();
}
