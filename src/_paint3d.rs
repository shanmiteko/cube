use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use xcb::x;
use xcb_util::ffi::image;

use crate::transform::transform;

const STATE_LEN: usize = 14;

pub type Point2 = (f64, f64);
pub type Segment2 = (Point2, Point2);
pub type Point3 = (f64, f64, f64);
pub type Segment3 = (Point3, Point3);

pub struct Paint3d {
    /// ```txt
    /// 0       1       2        3        4   5   6   7   8   9   10  11, 12, 13
    /// [width, height, event_x, event_y, dx, dy, dz, rx, ry, rz, sx, sy, sz, k]
    /// ```
    state: [f64; STATE_LEN],
    segments: Vec<Segment3>,
    window: x::Window,
    drawable: x::Drawable,
    gc: x::Gcontext,
    conn: xcb::Connection,
}

#[derive(Debug)]
pub enum InteractionEvent {
    Expose,
    Mouse(MouseEvent),
}

#[derive(Debug)]
pub enum MouseEvent<Move = (f64, f64, f64)> {
    /// (dx,dy,dz)
    LeftDrag(Move),
    /// (rx,ry,rz)
    RightDrag(Move),
    Scroll,
}

impl Paint3d {
    pub fn new(width: u16, height: u16) -> xcb::Result<Self> {
        let mut state = [0.; STATE_LEN];
        state[0] = width as f64;
        state[1] = height as f64;
        state[10] = (width + height) as f64 / 2.;
        state[11] = (width + height) as f64 / 2.;
        state[12] = (width + height) as f64 / 2.;
        state[13] = 2.;
        let (conn, screen_num) = xcb::Connection::connect(None).unwrap();
        let setup = conn.get_setup();
        let screen = setup.roots().nth(screen_num as usize).unwrap();

        let window: x::Window = conn.generate_id();
        conn.send_request(&x::CreateWindow {
            depth: screen.root_depth(),
            wid: window,
            parent: screen.root(),
            x: 0,
            y: 0,
            width,
            height,
            border_width: 10,
            class: x::WindowClass::InputOutput,
            visual: screen.root_visual(),
            value_list: &[
                x::Cw::BackPixel(screen.white_pixel()),
                x::Cw::EventMask(
                    x::EventMask::EXPOSURE
                        | x::EventMask::KEY_PRESS
                        | x::EventMask::BUTTON_PRESS
                        | x::EventMask::BUTTON_RELEASE
                        | x::EventMask::POINTER_MOTION,
                ),
            ],
        });

        let gc: x::Gcontext = conn.generate_id();
        conn.send_request(&x::CreateGc {
            cid: gc,
            drawable: x::Drawable::Window(window),
            value_list: &[x::Gc::Foreground(screen.black_pixel())],
        });

        conn.send_request(&x::MapWindow { window });

        conn.flush()?;

        Ok(Self {
            state,
            segments: vec![],
            window,
            drawable: x::Drawable::Window(window),
            gc,
            conn,
        })
    }

    pub fn test(&self) {
        image::xcb_image_create_native(c, width, height, format, depth, base, bytes, data)
    }

    pub fn segments(&mut self, segments: Vec<Segment3>) {
        for seg in segments {
            self.segments.push(seg);
        }
    }

    pub fn event_loop(mut self) -> xcb::Result<()> {
        loop {
            let event = match self.conn.wait_for_event() {
                Err(xcb::Error::Connection(xcb::ConnError::Connection)) => {
                    // graceful shutdown, likely "x" close button clicked in title bar
                    break Ok(());
                }
                Err(err) => {
                    panic!("unexpected error: {:#?}", err);
                }
                Ok(event) => event,
            };
            if let xcb::Event::X(xevent) = event {
                match xevent {
                    x::Event::Expose(ev) => {
                        let (width, height) = (ev.width(), ev.height());
                        if width > 1 && height > 1 {
                            self.state[0] = width as f64;
                            self.state[1] = height as f64;
                            self.event_handler(InteractionEvent::Expose)?;
                        }
                    }
                    x::Event::KeyPress(key_press) => {
                        if key_press.detail() == 0x18 {
                            // Q (on qwerty)
                            break Ok(());
                        }
                    }
                    x::Event::ButtonPress(btn_press) => {
                        match btn_press.detail() {
                            // 左键
                            // 右键
                            0x01 | 0x03 => {
                                self.state[2] = btn_press.event_x() as f64;
                                self.state[3] = btn_press.event_y() as f64;
                            }
                            // 向前滚轮
                            0x04 => {
                                match btn_press.state().is_empty() {
                                    true => {
                                        self.state[10] += 50.;
                                        self.event_handler(InteractionEvent::Mouse(
                                            MouseEvent::Scroll,
                                        ))?;
                                    }
                                    false => match btn_press.state() {
                                        // 带左键向前滚轮
                                        x::KeyButMask::BUTTON1 => {
                                            self.state[5] += 25.;
                                            self.event_handler(InteractionEvent::Mouse(
                                                MouseEvent::LeftDrag((
                                                    self.state[4],
                                                    self.state[5],
                                                    self.state[6],
                                                )),
                                            ))?;
                                        }
                                        // 带右键向前滚轮
                                        x::KeyButMask::BUTTON3 => {
                                            self.state[8] += 0.5;
                                            self.event_handler(InteractionEvent::Mouse(
                                                MouseEvent::RightDrag((
                                                    self.state[7],
                                                    self.state[8],
                                                    self.state[9],
                                                )),
                                            ))?;
                                        }
                                        _ => {}
                                    },
                                }
                            }
                            // 向后滚轮
                            0x05 => match btn_press.state().is_empty() {
                                true => {
                                    self.state[10] -= 50.;
                                    self.event_handler(InteractionEvent::Mouse(
                                        MouseEvent::Scroll,
                                    ))?;
                                }
                                false => match btn_press.state() {
                                    x::KeyButMask::BUTTON1 => {
                                        self.state[5] -= 25.;
                                        self.event_handler(InteractionEvent::Mouse(
                                            MouseEvent::LeftDrag((
                                                self.state[4],
                                                self.state[5],
                                                self.state[6],
                                            )),
                                        ))?;
                                    }
                                    x::KeyButMask::BUTTON3 => {
                                        self.state[8] -= 0.5;
                                        self.event_handler(InteractionEvent::Mouse(
                                            MouseEvent::RightDrag((
                                                self.state[7],
                                                self.state[8],
                                                self.state[9],
                                            )),
                                        ))?;
                                    }
                                    _ => {}
                                },
                            },
                            _ => {}
                        }
                    }
                    x::Event::ButtonRelease(btn_release) => match btn_release.detail() {
                        // 左键
                        0x01 => {
                            self.state[4] += btn_release.event_x() as f64 - self.state[2];
                            self.state[6] += btn_release.event_y() as f64 - self.state[3];
                        }
                        // 右键
                        0x03 => {
                            self.state[9] +=
                                (btn_release.event_x() as f64 - self.state[2]).to_radians();
                            self.state[7] +=
                                (btn_release.event_y() as f64 - self.state[3]).to_radians();
                        }
                        _ => {}
                    },
                    x::Event::MotionNotify(motion) => match motion.state() {
                        x::KeyButMask::BUTTON1 => {
                            self.event_handler(InteractionEvent::Mouse(MouseEvent::LeftDrag((
                                motion.event_x() as f64 - self.state[2] + self.state[4],
                                self.state[5],
                                motion.event_y() as f64 - self.state[3] + self.state[6],
                            ))))?;
                        }
                        x::KeyButMask::BUTTON3 => {
                            self.event_handler(InteractionEvent::Mouse(MouseEvent::RightDrag((
                                (motion.event_y() as f64 - self.state[3]).to_radians()
                                    + self.state[7],
                                self.state[8],
                                (motion.event_x() as f64 - self.state[2]).to_radians()
                                    + self.state[9],
                            ))))?;
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
    }

    fn event_handler(&self, ievent: InteractionEvent) -> xcb::Result<()> {
        match ievent {
            InteractionEvent::Expose => self.seg_paint(transform(
                &self.segments,
                (self.state[4], self.state[5], self.state[6]),
                (self.state[7], self.state[8], self.state[9]),
                (
                    (self.state[10], self.state[11], self.state[12]),
                    self.state[13],
                    self.state[0],
                    self.state[1],
                ),
            )),
            InteractionEvent::Mouse(mevent) => match mevent {
                MouseEvent::LeftDrag((dx, dy, dz)) => self.seg_paint(transform(
                    &self.segments,
                    (dx, dy, dz),
                    (self.state[7], self.state[8], self.state[9]),
                    (
                        (self.state[10], self.state[11], self.state[12]),
                        self.state[13],
                        self.state[0],
                        self.state[1],
                    ),
                )),
                MouseEvent::RightDrag((rx, ry, rz)) => self.seg_paint(transform(
                    &self.segments,
                    (self.state[4], self.state[5], self.state[6]),
                    (rx, ry, rz),
                    (
                        (self.state[10], self.state[11], self.state[12]),
                        self.state[13],
                        self.state[0],
                        self.state[1],
                    ),
                )),
                MouseEvent::Scroll => self.seg_paint(transform(
                    &self.segments,
                    (self.state[4], self.state[5], self.state[6]),
                    (self.state[7], self.state[8], self.state[9]),
                    (
                        (self.state[10], self.state[11], self.state[12]),
                        self.state[13],
                        self.state[0],
                        self.state[1],
                    ),
                )),
            },
        }
    }

    fn seg_paint(&self, segments: Vec<Segment2>) -> xcb::Result<()> {
        self.conn.send_request(&x::ClearArea {
            exposures: false,
            window: self.window,
            x: 0,
            y: 0,
            width: self.state[0] as u16,
            height: self.state[1] as u16,
        });
        self.conn.send_request(&x::PolySegment {
            drawable: self.drawable,
            gc: self.gc,
            segments: &segments
                .par_iter()
                .map(|((x1, y1), (x2, y2))| x::Segment {
                    x1: *x1 as i16,
                    y1: *y1 as i16,
                    x2: *x2 as i16,
                    y2: *y2 as i16,
                })
                .collect::<Vec<x::Segment>>(),
        });
        self.conn.flush()?;
        Ok(())
    }
}
