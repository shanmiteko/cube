mod xcbshow;

use xcbshow::XcbShow;

fn main() {
    let xcb_show = XcbShow::new(500, 400);

    'event_loop: loop {
        match xcb_show.events() {
            xcbshow::Event::Close => break 'event_loop,
            xcbshow::Event::Expose { width, height } => {
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
