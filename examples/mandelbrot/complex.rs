#[derive(Clone, Copy)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

impl std::ops::Add<Complex> for Complex {
    type Output = Complex;

    fn add(self, rhs: Complex) -> Self::Output {
        Complex::new(self.re + rhs.re, self.im + rhs.im)
    }
}

impl std::ops::Mul<Complex> for Complex {
    type Output = Complex;

    fn mul(self, rhs: Complex) -> Self::Output {
        Complex::new(
            self.re * rhs.re - self.im * rhs.im,
            self.re * rhs.im + self.im * rhs.re,
        )
    }
}

impl Complex {
    pub fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    pub fn sq_sum(&self) -> f64 {
        self.re * self.re + self.im * self.im
    }
}
