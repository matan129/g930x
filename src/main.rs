extern crate pcap;
extern crate libusb;

const VENDOR: u16 = 0x046d;
const PRODUCT: u16 = 0x0a1f;

fn main() {
    let ctx = libusb::Context::new().unwrap();
    let g930 = find_device(&ctx);

    match g930 {
        Some(device) => monitor_device(&device),
        None => println!("Failed to find G930, quitting...")
    }
}

fn find_device(ctx: &libusb::Context) -> Option<libusb::Device> {
    for device in ctx.devices().unwrap().iter() {
        let device_desc = device.device_descriptor().unwrap();

        if device_desc.vendor_id() == VENDOR
            && device_desc.product_id() == PRODUCT {
            println!("Found G930 at Bus {:03} Device {:03}", device.bus_number(), device.address());
            return Some(device);
        }
    }

    None
}

fn monitor_device(usb_device: &libusb::Device) {
    match find_usbmon().map(pcap::Device::open) {
        Some(Ok(mut cap)) => capture_device(&mut cap, &usb_device),
        Some(Err(e)) => println!("Failed to open G930 for capturing ({:?})", e),
        None => println!("Could not find USB monitor. Try modprobing for it.")
    }
}

fn find_usbmon() -> Option<pcap::Device> {
    for pcap_dev in pcap::Device::list().unwrap().into_iter() {
        if pcap_dev.name.contains("usbmon") {
            return Some(pcap_dev);
        }
    }

    None
}

fn capture_device(cap: &mut pcap::Capture<pcap::Active>, device: &libusb::Device) {
    let bpf = format!("ether[11:1] = {} and ether[12:1] = {} and ether[9:1] = 1 and ether[10:1] = 0x83",
                      device.address(),
                      device.bus_number());

    println!("Filtering with {}", &bpf);

    cap.filter(&bpf).unwrap();

    while let Ok(packet) = cap.next() {
        println!("received packet! {:?}", packet);
    }
}
