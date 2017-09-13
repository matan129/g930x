mod events;
mod usb_sniffer;
mod packet_parser;
mod packet_handler;
mod x11_handler;

use self::usb_sniffer::start_monitoring;

pub fn start() {
    start_monitoring()
}
