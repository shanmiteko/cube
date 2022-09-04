mod xcbshow;

use xcbshow::{InteractKind, XcbShow};

struct ImageBuf {
    width: usize,
    height: usize,
    raw_buf: Vec<u32>,
}

impl ImageBuf {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            raw_buf: vec![0xffffff; width * height],
        }
    }

    pub fn as_slice(&self) -> &[u32] {
        self.raw_buf.as_slice()
    }

    pub fn resize(&mut self, width: usize, height: usize) -> Self {
        let mut new_raw_buf = vec![0xffffff; width * height];
        new_raw_buf.iter_mut().enumerate().for_each(|(n, p)| {
            let pos = (n / width, n % width);
            if pos.0 < self.height && pos.1 < self.width {
                *p = self.raw_buf[pos.0 * self.width + pos.1]
            }
        });
        Self {
            width,
            height,
            raw_buf: new_raw_buf,
        }
    }
}

fn main() {
    let (width, height) = (500, 400);
    let xcb_show = XcbShow::new(width, height);

    let mut front = ImageBuf::new(width as usize, height as usize);
    front.raw_buf.fill(0xff0000);

    'event_loop: loop {
        match xcb_show.events() {
            xcbshow::Event::Close => break 'event_loop,
            xcbshow::Event::Expose { width, height } => {
                xcb_show.resize_image(width, height);

                let resize_front = front.resize(width as usize, height as usize);

                xcb_show.fill_image(resize_front.as_slice());
            }
            xcbshow::Event::Interact(interact_kind) => match interact_kind {
                _ => {}
            },
            xcbshow::Event::Noop => {}
        }
    }
}
