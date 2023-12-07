use num_complex::ComplexDistribution;
use rand::{
    distributions::{Distribution, Uniform},
    Rng,
};

pub type Complex = num_complex::Complex<f64>;

pub struct JuliaParameter;

impl Distribution<Complex> for JuliaParameter {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Complex {
        let dist = ComplexDistribution::new(Uniform::new(-1.5, 0.5), Uniform::new(-1.0, 1.0));
        dist.sample(rng)
    }
}
