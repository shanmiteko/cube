use crate::complex::Complex;

#[inline]
pub fn fcz(z: Complex, c: Complex) -> Complex {
    z * z + c
}

pub fn check(c: Complex, nmax: u32) -> (bool, u32) {
    let mut zn = Complex::new(0., 0.);
    for n in 0..nmax {
        if zn.sq_sum() > 4. {
            return (false, n);
        }
        zn = fcz(zn, c);
    }
    (true, nmax)
}

pub fn mandelbrot(x: i16, y: i16, scale: f64) -> u32 {
    match (x, y) {
        (0, 0) => 0xFF0000,
        _ => match check(Complex::new(x as f64 * scale, y as f64 * scale), 100) {
            (true, _) => 0x000000,
            (false, n) => 0xFFFFFF - n * 1000,
        },
    }
}
