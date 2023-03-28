use plotter::image::ImageBuf;
use plotter::xcbshow;

fn main() {
    let (width, height) = (500, 400);
    let mut xcb_show = xcbshow::XcbShow::new(width, height);

    let mut image_buf = ImageBuf::from_fn(width, height, |x, _y| match x {
        x if x < width / 3 => 0xFF0000,
        x if x > width / 3 && x < width * 2 / 3 => 0x00FF00,
        x if x > width * 2 / 3 && x < width => 0x0000FF,
        _ => 0,
    });

    'event_loop: loop {
        match xcb_show.events() {
            xcbshow::Event::Close => break 'event_loop,
            xcbshow::Event::Expose { width, height } => {
                xcb_show.resize_image(width, height);

                let resize_img = image_buf.resize(width, height);

                xcb_show.fill_image(resize_img.as_slice());
            }
            xcbshow::Event::Interact(_interact_kind) => {}
            xcbshow::Event::Noop => {}
        }
    }
}
