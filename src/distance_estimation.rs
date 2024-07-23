use cgmath::Zero;
use num_complex::ComplexDistribution;
use rand::Rng;
use rand_distr::{Distribution, Normal, Uniform};

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

        (magnitude / diff).sqrt() * 0.5 * magnitude.ln()
    }
}

pub struct MandelbrotBoundary {
    pub max_iter: usize,
}

impl MandelbrotBoundary {
    fn escape(&self) -> f64 {
        (self.max_iter as f64) * (self.max_iter as f64)
    }

    fn distance(&self, c: Complex) -> f64 {
        let mag = c.norm_sqr();
        if 256.0 * mag * mag - 96.0 * mag + 32.0 * c.re - 3.0 < 0.0 {
            return 0.0;
        }

        let mut z = Complex::zero();
        let mut dz = Complex::zero();

        let escape = self.escape();

        for _ in 0..self.max_iter {
            dz = 2.0 * z * dz + 1.0;
            z = z * z + c;

            let mag = z.norm_sqr();
            if mag > escape {
                let dmag = dz.norm_sqr();
                return 0.5 * (mag / dmag).sqrt() * mag.ln();
            }
        }

        0.0
    }
}

impl Distribution<Complex> for MandelbrotBoundary {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Complex {
        const DISTANCE: f64 = 1e-3;

        let bbx_dist = ComplexDistribution::new(Uniform::new(-2.0, 0.5), Uniform::new(-1.2, 1.2));
        let c_preferred = rng
            .sample_iter(bbx_dist)
            .find(|&c| {
                let dist = self.distance(c);
                0.0 < dist && dist < DISTANCE
            })
            .unwrap();

        let r: f64 = rng.sample(Normal::new(0.0, 40.0 * DISTANCE).unwrap());
        let theta = rng.gen_range(0.0..std::f64::consts::TAU);
        let pertubation = Complex::from_polar(r, theta);

        c_preferred + pertubation
    }
}
