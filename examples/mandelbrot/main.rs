use mandelbrot::mandelbrot;
use plotter::image::ImageBuf;
use plotter::xcbshow;

mod complex;
mod mandelbrot;

fn main() {
    let (mut width, mut height) = (500, 500);
    let mut xcb_show = xcbshow::XcbShow::new(width, height);

    let base_scale = 1. / height as f64;
    let mut scale = base_scale;

    let mut base_pos = (0i16, 0i16);
    let mut last_pos = (0i16, 0i16);
    let mut wheel = 0i16;
    'event_loop: loop {
        match xcb_show.events() {
            xcbshow::Event::Close => break 'event_loop,
            xcbshow::Event::Expose {
                width: w,
                height: h,
            } => {
                width = w;
                height = h;
                xcb_show.resize_image(width, height);
                xcb_show.fill_image(
                    ImageBuf::from_fn(width, height, |x, y| mandelbrot(x as i16, y as i16, scale))
                        .as_slice(),
                );
            }
            xcbshow::Event::Interact(interact_kind) => match interact_kind {
                xcbshow::InteractKind::LeftPress { state: _, pos } => last_pos = pos,
                xcbshow::InteractKind::LeftRelease { state: _, pos } => {
                    if pos != base_pos {
                        xcb_show.resize_image(width, height);
                        xcb_show.fill_image(
                            ImageBuf::from_fn(width, height, |x, y| {
                                mandelbrot(
                                    x as i16 - pos.0 + last_pos.0 + base_pos.0,
                                    y as i16 - pos.1 + last_pos.1 + base_pos.1,
                                    scale,
                                )
                            })
                            .as_slice(),
                        );
                        base_pos = (
                            base_pos.0 - pos.0 + last_pos.0,
                            base_pos.1 - pos.1 + last_pos.1,
                        );
                    }
                }
                xcbshow::InteractKind::Wheel { state: _, step } => {
                    wheel += step;
                    if wheel % 2 == 0 {
                        if wheel < 0 {
                            scale = 1. / wheel.abs() as f64 * base_scale;
                        } else {
                            scale = wheel as f64 * base_scale;
                        }
                        xcb_show.resize_image(width, height);
                        xcb_show.fill_image(
                            ImageBuf::from_fn(width, height, |x, y| {
                                mandelbrot(x as i16 + base_pos.0, y as i16 + base_pos.1, scale)
                            })
                            .as_slice(),
                        );
                    }
                }
                _ => (),
            },
            xcbshow::Event::Noop => {}
        }
    }
}
