mod ffi {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

type Step = i16;
type Pos = (Step, Step);

pub enum Event {
    Noop,
    Close,
    Expose { width: u16, height: u16 },
    Interact(InteractKind),
}

#[derive(Debug)]
pub enum InteractKind {
    KeyPress { state: InteractDevice, key: u8 },
    LeftPress { state: InteractDevice, pos: Pos },
    LeftRelease { state: InteractDevice, pos: Pos },
    Wheel { state: InteractDevice, step: Step },
    RightPress { state: InteractDevice, pos: Pos },
    RightRelease { state: InteractDevice, pos: Pos },
    Move { state: InteractDevice, pos: Pos },
}

#[derive(Debug)]
pub enum InteractDevice {
    Mouse(Mouse),
    KeyBoard(u16),
}

#[derive(Debug)]
pub enum Mouse {
    Left,
    Right,
    Wheel,
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

    pub fn resize_image(&self, width: u16, height: u16) {
        unsafe {
            ffi::resize_image(self.raw_window, self.raw_image, width, height);
        }
    }

    fn show_image(&self) {
        unsafe {
            ffi::show_image(self.raw_window, self.raw_image);
        }
    }

    /// `memcopy`
    pub fn fill_image(&self, pixels: &[u32]) {
        unsafe {
            assert_eq!(
                pixels.len(),
                (*self.raw_image).pixel_count as usize,
                "the length of pixels should be equal to pixel_count"
            );
            ffi::update_image(self.raw_image, pixels.as_ptr());
        }
        self.show_image();
    }

    pub fn events(&mut self) -> Event {
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
                3 => match state {
                    256 => Event::Interact(InteractKind::Move {
                        state: InteractDevice::Mouse(Mouse::Left),
                        pos: (x, y),
                    }),
                    512 => Event::Interact(InteractKind::Move {
                        state: InteractDevice::Mouse(Mouse::Wheel),
                        pos: (x, y),
                    }),
                    1024 => Event::Interact(InteractKind::Move {
                        state: InteractDevice::Mouse(Mouse::Right),
                        pos: (x, y),
                    }),
                    _ => Event::Noop,
                },
                4 => match detail {
                    1 => Event::Interact(InteractKind::LeftPress {
                        state: InteractDevice::KeyBoard(state),
                        pos: (x, y),
                    }),
                    2 => Event::Interact(InteractKind::Wheel {
                        state: InteractDevice::KeyBoard(state),
                        step: 0,
                    }),
                    3 => Event::Interact(InteractKind::RightPress {
                        state: InteractDevice::KeyBoard(state),
                        pos: (x, y),
                    }),
                    4 => Event::Interact(InteractKind::Wheel {
                        state: InteractDevice::KeyBoard(state),
                        step: 1,
                    }),
                    5 => Event::Interact(InteractKind::Wheel {
                        state: InteractDevice::KeyBoard(state),
                        step: -1,
                    }),
                    _ => Event::Noop,
                },
                5 => match detail {
                    1 => Event::Interact(InteractKind::LeftRelease {
                        state: InteractDevice::KeyBoard(state),
                        pos: (x, y),
                    }),
                    2 => Event::Interact(InteractKind::Wheel {
                        state: InteractDevice::KeyBoard(state),
                        step: 0,
                    }),
                    3 => Event::Interact(InteractKind::RightRelease {
                        state: InteractDevice::KeyBoard(state),
                        pos: (x, y),
                    }),
                    4 => Event::Interact(InteractKind::Wheel {
                        state: InteractDevice::KeyBoard(state),
                        step: 1,
                    }),
                    5 => Event::Interact(InteractKind::Wheel {
                        state: InteractDevice::KeyBoard(state),
                        step: -1,
                    }),
                    _ => Event::Noop,
                },
                6 => Event::Interact(InteractKind::KeyPress {
                    key: detail,
                    state: InteractDevice::KeyBoard(state),
                }),
                _ => Event::Noop,
            }
        }
    }
}
