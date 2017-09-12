use libusb::Context;
use libusb::Device as UsbDevice;

use pcap::Device as PcapDevice;
use pcap::{Active, Capture, Packet};
use x11_dl::{xlib, keysym};
use std::ptr;
use std::os::raw::{c_int, c_uint, c_ulong, c_uchar};

const VENDOR: u16 = 0x046d;
const PRODUCT: u16 = 0x0a1f;

pub fn start() {
    let ctx = Context::new().unwrap();
    match find_device(&ctx) {
        Some(device) => monitor_device(&device),
        None => println!("Failed to find G930, quitting...")
    };
}

fn find_device(ctx: &Context) -> Option<UsbDevice> {
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

fn monitor_device(usb_device: &UsbDevice) {
    match find_usbmon().map(PcapDevice::open) {
        Some(Ok(mut cap)) => capture_device(&mut cap, &usb_device),
        Some(Err(e)) => println!("Failed to open G930 for capturing ({:?})", e),
        None => println!("Could not find USB monitor. Try modprobing for it.")
    };
}

fn find_usbmon() -> Option<PcapDevice> {
    for pcap_dev in PcapDevice::list().unwrap().into_iter() {
        if pcap_dev.name.contains("usbmon") {
            return Some(pcap_dev);
        }
    }

    None
}

fn capture_device(cap: &mut Capture<Active>, device: &UsbDevice) {
    let bpf = format!("len > 64 and ether[11:1] = {} and ether[12:1] = {} and ether[9:1] = 1 and ether[10:1] = 0x83",
                      device.address(),
                      device.bus_number());

    println!("Filtering with {}", &bpf);
    cap.filter(&bpf).unwrap();

    let mut handler = X11Handler::new();

    while let Ok(packet) = cap.next() {
        handler.handle(&packet).unwrap();
    };
}

trait PacketHandler {
    fn new() -> Self;

    #[allow(unused_variables)]
    fn handle(&mut self, packet: &Packet) -> Result<(), &'static str> {
        Ok(())
    }
}

struct X11Handler {
    xlib: xlib::Xlib,
    display: *mut xlib::Display,
    keydown_flag: bool
}

impl X11Handler {
    fn create_key_event(&mut self, window: xlib::Window, root_window: xlib::Window, keydown: bool, keycode: c_uint) -> xlib::XKeyEvent {

        let event_type: c_int = if keydown {
            xlib::KeyPress
        } else {
            xlib::KeyRelease
        };

        let actual_keycode = unsafe {
            (self.xlib.XKeysymToKeycode)(self.display, keycode as u64) as u32
        };

        xlib::XKeyEvent {
            type_: event_type,
            serial: 0 as c_ulong,
            send_event: xlib::False,
            display: self.display,
            window,
            root: root_window,
            subwindow: 0,
            time: xlib::CurrentTime,
            x: 1,
            y: 1,
            x_root: 1,
            y_root: 1,
            state: 0,
            keycode: actual_keycode,
            same_screen: xlib::True,
        }
    }
}

impl PacketHandler for X11Handler {
    fn new() -> Self {
        let xlib: xlib::Xlib = xlib::Xlib::open().unwrap();
        let display: *mut xlib::Display = unsafe {
            let display = (xlib.XOpenDisplay)(ptr::null());
            if display.is_null() {
                panic!("XOpenDisplay failed");
            }

            display
        };

        X11Handler {
            xlib,
            display,
            keydown_flag: true
        }
    }

    fn handle(&mut self, packet: &Packet) -> Result<(), &'static str> {
        println!("handling packet {:?}. flag {}", packet, self.keydown_flag);

        let root_window = unsafe {
            (self.xlib.XDefaultRootWindow)(self.display)
        };

        let focused_window = unsafe {
            let focused_window_ptr = Box::new(0 as c_ulong);
            let raw_focused_window_ptr = Box::into_raw(focused_window_ptr);
            let revert_ptr = Box::new(0 as c_int);
            let raw_revert_ptr = Box::into_raw(revert_ptr);
            (self.xlib.XGetInputFocus)(self.display, raw_focused_window_ptr, raw_revert_ptr);

            Box::from_raw(raw_revert_ptr);
            *Box::from_raw(raw_focused_window_ptr)
        };

        let keydown = self.keydown_flag;
        let event = self.create_key_event(focused_window, root_window, keydown, keysym::XK_space);
        let event_ptr = Box::new(xlib::XEvent::from(event));

        unsafe {
            let raw_event_ptr = Box::into_raw(event_ptr);
            (self.xlib.XSendEvent)(self.display, focused_window, xlib::True, xlib::KeyPressMask, raw_event_ptr);
            Box::from_raw(raw_event_ptr);
        }

        self.keydown_flag = !self.keydown_flag;
        Ok(())
    }
}

impl Drop for X11Handler {
    fn drop(&mut self) {
        unsafe {
            (self.xlib.XCloseDisplay)(self.display);
        }
    }
}
