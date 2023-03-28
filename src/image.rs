use rayon::prelude::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

pub struct ImageBuf {
    width: u16,
    height: u16,
    raw_buf: Vec<u32>,
}

impl ImageBuf {
    const BACKGROUND: u32 = 0xffffff;

    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            raw_buf: vec![Self::BACKGROUND; width as usize * height as usize],
        }
    }

    pub fn from_fn<F>(width: u16, height: u16, f: F) -> Self
    where
        F: Fn(u16, u16) -> u32 + std::marker::Sync,
    {
        let mut img = Self::new(width, height);
        img.raw_buf.par_iter_mut().enumerate().for_each(|(n, p)| {
            let (x, y) = (n % width as usize, n / width as usize);
            *p = f(x as u16, y as u16);
        });
        img
    }

    pub fn as_slice(&self) -> &[u32] {
        self.raw_buf.as_slice()
    }

    pub fn resize(&mut self, width: u16, height: u16) -> Self {
        Self::from_fn(width, height, |x, y| {
            if x < self.width && y < self.height {
                self.raw_buf[y as usize * self.width as usize + x as usize]
            } else {
                Self::BACKGROUND
            }
        })
    }
}
