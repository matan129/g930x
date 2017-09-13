mod packet_handler;
mod x11_handler;
mod usb_sniffer;

use self::usb_sniffer::start_monitoring;

pub fn start() {
    start_monitoring()
}
