use crate::complex::Complex;
use rand::Rng;
use rand::distr::Distribution;

struct Sign;

impl Distribution<f64> for Sign {
    #[inline]
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> f64 {
        if rng.random() { 1.0 } else { -1.0 }
    }
}

struct RandomSqrt(Complex);

impl Distribution<Complex> for RandomSqrt {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Complex {
        self.0.sqrt() * rng.sample(Sign)
    }
}

pub struct InverseIteration {
    pub c: Complex,
    rng: rand::rngs::ThreadRng,
}

impl InverseIteration {
    pub fn new(c: Complex) -> Self {
        Self {
            c,
            rng: rand::rng(),
        }
    }

    fn random_sqrt(&mut self, x: Complex) -> Complex {
        self.rng.sample(RandomSqrt(x))
    }

    fn fixpoint(&mut self) -> Complex {
        self.random_sqrt(0.25 - self.c) + 0.5
    }

    fn preimage(&mut self, point: Complex) -> Complex {
        self.random_sqrt(point - self.c)
    }
}

pub struct InverseIterator {
    julia: InverseIteration,
    point: Complex,
}

impl Iterator for InverseIterator {
    type Item = Complex;

    fn next(&mut self) -> Option<Self::Item> {
        let point = self.point;

        self.point = self.julia.preimage(point);

        Some(point)
    }
}

impl IntoIterator for InverseIteration {
    type Item = Complex;

    type IntoIter = InverseIterator;

    fn into_iter(mut self) -> Self::IntoIter {
        let point = self.fixpoint();
        Self::IntoIter { julia: self, point }
    }
}
