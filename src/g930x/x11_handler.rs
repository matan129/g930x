use x11::{xlib, keysym};
use x11::xlib::{Display, Window, XEvent, XKeyEvent, KeyPress, KeyRelease};
use std::ptr;
use std::os::raw::{c_int, c_uint, c_ulong};
use g930x::events::{G930KeyEvent, G930Keys, G930KeyEventTypes};
use g930x::packet_handler::EventHandler;

pub struct X11Handler {
    display: *mut Display,
    last_keycode: Option<c_uint>
}

impl X11Handler {
    fn open_x11_display() -> *mut Display {
        unsafe {
            let display = (xlib::XOpenDisplay)(ptr::null());
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
            (xlib::XGetInputFocus)(self.display, raw_focused_window_ptr, raw_revert_ptr);

            Box::from_raw(raw_revert_ptr);
            *Box::from_raw(raw_focused_window_ptr)
        }
    }

    fn find_root_window(&self) -> Window {
        unsafe {
            (xlib::XDefaultRootWindow)(self.display)
        }
    }

    fn keysym_to_keycode(&self, keycode: c_uint) -> u32 {
        unsafe {
            (xlib::XKeysymToKeycode)(self.display, keycode as u64) as u32
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
            (xlib::XSendEvent)(self.display, window, xlib::True, xlib::KeyPressMask, raw_event_ptr);
            Box::from_raw(raw_event_ptr);
        }
    }
}

impl EventHandler for X11Handler {
    fn new() -> Self {
        let display = Self::open_x11_display();

        X11Handler {
            display,
            last_keycode: None
        }
    }

    fn handle(&mut self, event: &G930KeyEvent) -> Result<(), &'static str> {
        println!("handling {:?}.", event);

        let keydown = event.type_ == G930KeyEventTypes::KeyDown;
        let focused_window = self.find_focused_window();
        let keycode_opt = match event.key {
            G930Keys::G1 => Some(keysym::XK_Next),
            G930Keys::G2 => Some(keysym::XK_space),
            G930Keys::G3 => Some(keysym::XK_BackSpace),
            G930Keys::UNKNOWN => self.last_keycode
        };

        keycode_opt.map(|keycode| {
            self.last_keycode = Some(keycode);
            let event = self.create_key_event(focused_window, keydown, keycode);
            self.send_event_to_window(event, focused_window);
        });

        Ok(())
    }
}

impl Drop for X11Handler {
    fn drop(&mut self) {
        unsafe {
            (xlib::XCloseDisplay)(self.display);
        }
    }
}
