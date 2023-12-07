use crate::complex::Complex;

pub struct DistanceEstimation {
    c: Complex,
}

impl DistanceEstimation {
    pub fn new(c: Complex) -> Self {
        Self { c }
    }

    pub fn distance(&self, mut z: Complex, max_iter: usize) -> f64 {
        // Squared norm of iteraded point z.
        let mut magnitude = z.norm_sqr();

        // Derivative of the magnitude of z, squared.
        let mut diff = 1.0;

        let escape = max_iter as f64 * max_iter as f64;

        for _ in 0..max_iter {
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
