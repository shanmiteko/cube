mod xcbshow;

use xcbshow::{InteractKind, XcbShow};

fn main() {
    let xcb_show = XcbShow::new(500, 400);

    'event_loop: loop {
        match xcb_show.events() {
            xcbshow::Event::Close => break 'event_loop,
            xcbshow::Event::Expose { width, height } => {
                xcb_show.resize_image(width, height);
                xcb_show.modify_image(|(r, c), pixel| {
                    if r == c {
                        *pixel = 0x0000ff;
                    } else {
                        *pixel = 0xffffff;
                    }
                });
                xcb_show.show_image();
            }
            xcbshow::Event::Interact(interact_kind) => match interact_kind {
                InteractKind::Move { state: _, pos } => {
                    xcb_show.modify_image(|(r, c), pixel| {
                        let lambda = |x: f64| 0.002 * (x - pos.0 as f64).powi(2) + pos.1 as f64;
                        let arclambda = |y: f64| ((y - pos.1 as f64) / 0.002).sqrt() + pos.0 as f64;
                        if lambda(c as f64).floor() == r as f64
                            || lambda(c as f64 + 1.).floor() == r as f64
                            || arclambda(r as f64).floor() == c as f64
                            || arclambda(r as f64 + 1.).floor() == c as f64
                        {
                            *pixel = 0x0000ff;
                        } else {
                            *pixel = 0xffffff;
                        }
                    });
                    xcb_show.show_image();
                }
                _ => {}
            },
            xcbshow::Event::Noop => {}
        }
    }
}
