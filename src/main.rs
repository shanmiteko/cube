mod xcbshow;

use xcbshow::XcbShow;

fn main() {
    let xcb_show = XcbShow::new(500, 400);

    'event_loop: loop {
        match xcb_show.events() {
            xcbshow::Event::Close => break 'event_loop,
            xcbshow::Event::Expose { width, height } => {
                xcb_show.resize_image(width, height);
                xcb_show.modify_image(|(r, c), pixel| {
                    let lambda = |x: f64| 0.002 * x * x;
                    let arclambda = |y: f64| (y / 0.002).sqrt();
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
                println!("w-{width} h-{height}")
            }
            xcbshow::Event::Interact {
                x,
                y,
                state,
                detail,
            } => {
                println!("x-{x} y-{y} state-{state} detail-{detail}")
            }
            xcbshow::Event::Noop => {}
        }
    }
}
