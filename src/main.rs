use xcb::x;

fn main() -> xcb::Result<()> {
    let (conn, screen_num) = xcb::Connection::connect(None).unwrap();
    let setup = conn.get_setup();
    let screen = setup.roots().nth(screen_num as usize).unwrap();

    let gc: x::Gcontext = conn.generate_id();

    let window: x::Window = conn.generate_id();
    let (width, height) = (300, 300);
    conn.send_request(&x::CreateWindow {
        depth: x::COPY_FROM_PARENT as u8,
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

    conn.send_request(&x::MapWindow { window });

    conn.send_request(&x::CreateGc {
        cid: gc,
        drawable: x::Drawable::Window(window),
        value_list: &[
            x::Gc::Foreground(screen.black_pixel()),
            x::Gc::GraphicsExposures(false),
        ],
    });

    conn.flush()?;

    let (cl, cw, ch, mut d1, d2, mut w, mut h, mut dx, mut dz) = (
        300.,
        300.,
        300.,
        300.,
        300.,
        width as f64,
        height as f64,
        0.,
        0.,
    );

    let mut cursor = (0, 0);

    loop {
        let event = match conn.wait_for_event() {
            Err(xcb::Error::Connection(xcb::ConnError::Connection)) => {
                // graceful shutdown, likely "x" close button clicked in title bar
                break Ok(());
            }
            Err(err) => {
                panic!("unexpected error: {:#?}", err);
            }
            Ok(event) => event,
        };
        match event {
            xcb::Event::X(xevent) => match xevent {
                x::Event::Expose(ev) => {
                    (w, h) = (ev.width() as f64, ev.height() as f64);

                    if w > 1.0 && h > 1.0 {
                        conn.send_request(&x::PolyLine {
                            coordinate_mode: x::CoordMode::Origin,
                            drawable: x::Drawable::Window(window),
                            gc,
                            points: &cube(cl, cw, ch, d1, d2, w, h, dx, dz),
                        });
                        conn.flush()?;
                    }
                }
                x::Event::KeyPress(key_press) => {
                    if key_press.detail() == 0x18 {
                        // Q (on qwerty)
                        break Ok(());
                    }
                }
                x::Event::ButtonPress(btn_press) => match btn_press.detail() {
                    0x01 => {
                        cursor = (btn_press.event_x(), btn_press.event_y());
                    }
                    0x04 => {
                        d1 -= 50.;
                        conn.send_request(&x::ClearArea {
                            exposures: false,
                            window,
                            x: 0,
                            y: 0,
                            width: w as u16,
                            height: h as u16,
                        });
                        conn.send_request(&x::PolyLine {
                            coordinate_mode: x::CoordMode::Origin,
                            drawable: x::Drawable::Window(window),
                            gc,
                            points: &cube(cl, cw, ch, d1, d2, w, h, dx, dz),
                        });
                        conn.flush()?;
                    }
                    0x05 => {
                        d1 += 50.;
                        conn.send_request(&x::ClearArea {
                            exposures: false,
                            window,
                            x: 0,
                            y: 0,
                            width: w as u16,
                            height: h as u16,
                        });
                        conn.send_request(&x::PolyLine {
                            coordinate_mode: x::CoordMode::Origin,
                            drawable: x::Drawable::Window(window),
                            gc,
                            points: &cube(cl, cw, ch, d1, d2, w, h, dx, dz),
                        });
                        conn.flush()?;
                    }
                    _ => {}
                },
                x::Event::ButtonRelease(btn_release) => match btn_release.detail() {
                    0x01 => {
                        dx += (btn_release.event_x() - cursor.0) as f64;
                        dz += -(btn_release.event_y() - cursor.1) as f64;
                    }
                    _ => {}
                },
                x::Event::MotionNotify(motion) => match motion.state() {
                    x::KeyButMask::BUTTON1 => {
                        conn.send_request(&x::ClearArea {
                            exposures: false,
                            window,
                            x: 0,
                            y: 0,
                            width: w as u16,
                            height: h as u16,
                        });
                        conn.send_request(&x::PolyLine {
                            coordinate_mode: x::CoordMode::Origin,
                            drawable: x::Drawable::Window(window),
                            gc,
                            points: &cube(
                                cl,
                                cw,
                                ch,
                                d1,
                                d2,
                                w,
                                h,
                                dx + (motion.event_x() - cursor.0) as f64,
                                dz - (motion.event_y() - cursor.1) as f64,
                            ),
                        });
                        conn.flush()?;
                    }
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        }
    }
}

macro_rules! shadow {
    ($a: expr, $b: expr, $c: expr, $d1: expr, $d2:expr, $w: expr, $h: expr) => {
        xcb::x::Point {
            x: ((2. * $a * $d2 + $w * $d1 + $w * $d2 - $w * $b) / (2. * $d1 + 2. * $d2 - 2. * $b))
                as i16,
            y: (($h * $d1 + $h * $d2 - $b * $h - 2. * $c * $d2) / (2. * $d1 + 2. * $d2 - 2. * $b))
                as i16,
        }
    };
}

macro_rules! polyline {
    () => {
        vec![]
    };
    ($expr:expr) => {
        $expr
    };
    ($($expr:expr,)*) => {
        vec![$(
            $expr,
        )*]
    };
    (($x:literal,$y:literal)) => {
        vec![xcb::x::Point { x: $x, y: $y }]
    };
    ($(($x:literal,$y:literal),)*) => {
        vec![$(
            xcb::x::Point { x: $x, y: $y },
        )*]
    };
}

fn cube(
    cl: f64,
    cw: f64,
    ch: f64,
    d1: f64,
    d2: f64,
    w: f64,
    h: f64,
    dx: f64,
    dz: f64,
) -> Vec<x::Point> {
    let (a0, b0, c0, d0, a1, b1, c1, d1) = (
        shadow!(-cl / 2. + dx, cw / 2., ch / 2. + dz, d1, d2, w, h),
        shadow!(cl / 2. + dx, cw / 2., ch / 2. + dz, d1, d2, w, h),
        shadow!(cl / 2. + dx, cw / 2., -ch / 2. + dz, d1, d2, w, h),
        shadow!(-cl / 2. + dx, cw / 2., -ch / 2. + dz, d1, d2, w, h),
        shadow!(-cl / 2. + dx, -cw / 2., ch / 2. + dz, d1, d2, w, h),
        shadow!(cl / 2. + dx, -cw / 2., ch / 2. + dz, d1, d2, w, h),
        shadow!(cl / 2. + dx, -cw / 2., -ch / 2. + dz, d1, d2, w, h),
        shadow!(-cl / 2. + dx, -cw / 2., -ch / 2. + dz, d1, d2, w, h),
    );
    polyline!(a0, b0, c0, d0, a0, a1, b1, c1, d1, a1, a1, a0, d0, d1, a1, b1, b0, c0, c1, b1,)
}
