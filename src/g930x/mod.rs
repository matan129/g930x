use libusb::Context;
use libusb::Device as UsbDevice;

use pcap::Device as PcapDevice;
use pcap::{Active, Capture, Packet};
use x11_dl::{xlib, keysym};
use x11_dl::xlib::{Xlib, Display, Window, XEvent, XKeyEvent, KeyPress, KeyRelease};
use std::ptr;
use std::os::raw::{c_int, c_uint, c_ulong};

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
    fn handle(&mut self, packet: &Packet) -> Result<(), &'static str>;
}

struct X11Handler {
    xlib: Xlib,
    display: *mut Display,
    keydown_flag: bool
}

impl X11Handler {
    fn open_x11_display(xlib: &Xlib) -> *mut Display {
        unsafe {
            let display = (xlib.XOpenDisplay)(ptr::null());
            if display.is_null() {
                panic!("XOpenDisplay failed");
            }

            display
        }
    }

    fn find_focused_window(&self) -> Window {
        unsafe {
            let focused_window_ptr = Box::new(0 as c_ulong);
            let raw_focused_window_ptr = Box::into_raw(focused_window_ptr);
            let revert_ptr = Box::new(0 as c_int);
            let raw_revert_ptr = Box::into_raw(revert_ptr);
            (self.xlib.XGetInputFocus)(self.display, raw_focused_window_ptr, raw_revert_ptr);

            Box::from_raw(raw_revert_ptr);
            *Box::from_raw(raw_focused_window_ptr)
        }
    }

    fn find_root_window(&self) -> Window {
        unsafe {
            (self.xlib.XDefaultRootWindow)(self.display)
        }
    }

    fn keysym_to_keycode(&self, keycode: c_uint) -> u32 {
        unsafe {
            (self.xlib.XKeysymToKeycode)(self.display, keycode as u64) as u32
        }
    }

    fn create_key_event(&self, window: Window, keydown: bool, keycode: c_uint) -> XKeyEvent {
        let event_type = if keydown {
            KeyPress
        } else {
            KeyRelease
        };

        XKeyEvent {
            type_: event_type,
            serial: 0 as c_ulong,
            send_event: xlib::False,
            display: self.display,
            window,
            root: self.find_root_window(),
            subwindow: 0,
            time: xlib::CurrentTime,
            x: 1,
            y: 1,
            x_root: 1,
            y_root: 1,
            state: 0,
            keycode: self.keysym_to_keycode(keycode),
            same_screen: xlib::True,
        }
    }

    fn event_to_boxed(event: XKeyEvent) -> Box<XEvent> {
        Box::new(XEvent::from(event))
    }

    fn send_event_to_window(&self, event: XKeyEvent, window: Window) {
        let event_ptr = Self::event_to_boxed(event);

        unsafe {
            let raw_event_ptr = Box::into_raw(event_ptr);
            (self.xlib.XSendEvent)(self.display, window, xlib::True, xlib::KeyPressMask, raw_event_ptr);
            Box::from_raw(raw_event_ptr);
        }
    }
}

impl PacketHandler for X11Handler {
    fn new() -> Self {
        let xlib = Xlib::open().unwrap();
        let display = Self::open_x11_display(&xlib);

        X11Handler {
            xlib,
            display,
            keydown_flag: true
        }
    }

    fn handle(&mut self, packet: &Packet) -> Result<(), &'static str> {
        println!("handling packet {:?}. flag {}", packet, self.keydown_flag);

        let keydown = self.keydown_flag; // TODO: actually parse the packet
        let focused_window = self.find_focused_window();
        let event = self.create_key_event(focused_window, keydown, keysym::XK_space);
        self.send_event_to_window(event, focused_window);

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
