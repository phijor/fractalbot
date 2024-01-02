use cgmath::Zero;

use crate::complex::Complex;

pub struct DistanceEstimation {
    c: Complex,
    max_iter: usize,
}

impl DistanceEstimation {
    pub fn new(c: Complex, max_iter: usize) -> Self {
        Self { c, max_iter }
    }

    /// A Julia set is connected if and only if the orbit
    /// of the critical point 0+0i is bounded.
    ///
    /// In other words: a Julia set is connected if c lies in
    /// the Mandelbrot set.
    pub fn is_connected(&self) -> bool {
        let mut z = Complex::zero();

        for _ in 0..self.max_iter {
            z = z * z + self.c;
            if z.norm_sqr() > 4.0 {
                return false;
            }
        }
        true
    }

    pub fn distance(&self, mut z: Complex) -> f64 {
        // Squared norm of iteraded point z.
        let mut magnitude = z.norm_sqr();

        // Derivative of the magnitude of z, squared.
        let mut diff = 1.0;

        let max_iter_f = self.max_iter as f64;
        let escape = max_iter_f * max_iter_f;

        for _ in 0..self.max_iter {
            diff *= 4.0 * magnitude;
            z = z * z + self.c;

            magnitude = z.norm_sqr();

            if magnitude > escape {
                break;
            }
        }

        // Resolve singularities of the distance estimation:
        // d(magnitude) = sqrt(magnitude/magnitude') * ln(magnitude)
        //
        // d(magnitude) → 0 as magnitude → 0
        if diff < f64::EPSILON || magnitude < f64::EPSILON {
            return 0.0;
        }

        (magnitude / diff).sqrt() * magnitude.ln()
    }
}
