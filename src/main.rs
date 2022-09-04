mod xcbshow;

use xcbshow::{InteractKind, XcbShow};

fn main() {
    let (width, height) = (500, 400);
    let xcb_show = XcbShow::new(width, height);

    let mut front_buf = vec![0xffffff; width as usize * height as usize];

    'event_loop: loop {
        match xcb_show.events() {
            xcbshow::Event::Close => break 'event_loop,
            xcbshow::Event::Expose { width, height } => {
                xcb_show.resize(width, height);

                front_buf.resize(width as usize * height as usize, 0xff0000);

                xcb_show.fill_image(&front_buf);
            }
            xcbshow::Event::Interact(interact_kind) => match interact_kind {
                _ => {}
            },
            xcbshow::Event::Noop => {}
        }
    }
}
