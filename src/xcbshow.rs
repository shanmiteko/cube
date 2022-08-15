mod ffi {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub enum Event {
    Noop,
    Close,
    Expose {
        width: u16,
        height: u16,
    },
    Interact {
        x: i16,
        y: i16,
        state: u16,
        detail: u8,
    },
}

pub struct XcbShow {
    raw_window: *mut ffi::window_t,
    raw_image: *mut ffi::image_t,
}

impl Drop for XcbShow {
    fn drop(&mut self) {
        unsafe {
            ffi::destroy_image(self.raw_image);
            ffi::destroy_window(self.raw_window);
        }
    }
}

impl XcbShow {
    pub fn new(width: u16, height: u16) -> Self {
        let raw_window = unsafe { ffi::create_window(width, height) };
        let raw_image = unsafe { ffi::create_image(raw_window, width, height) };
        Self {
            raw_window,
            raw_image,
        }
    }

    pub fn events(&self) -> Event {
        unsafe {
            let raw_event = ffi::wait_for_event(self.raw_window);
            let ffi::event_t {
                width,
                height,
                x,
                y,
                state,
                detail,
                kind,
            } = *raw_event;
            ffi::destroy_event(raw_event);
            match kind {
                1 => Event::Close,
                2 => match (width, height) {
                    (w, h) if w > 1 && h > 1 => Event::Expose { width, height },
                    _ => Event::Noop,
                },
                3 => Event::Interact {
                    x,
                    y,
                    state,
                    detail,
                },
                _ => Event::Noop,
            }
        }
    }
}
