use pcap::Packet;
use x11_dl::{xlib, keysym};
use x11_dl::xlib::{Xlib, Display, Window, XEvent, XKeyEvent, KeyPress, KeyRelease};
use std::ptr;
use std::os::raw::{c_int, c_uint, c_ulong};

use g930x::packet_handler::PacketHandler;

pub struct X11Handler {
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
