use cgmath::{prelude::*, vec3, Vector3};
use rand::{distributions::Distribution, seq::SliceRandom};
use rand_distr::{Pert, Uniform};

type Vec3 = Vector3<f64>;

// https://iquilezles.org/articles/palettes/
fn palette_vec(t: f64, a: &Vec3, b: &Vec3, c: &Vec3, d: &Vec3) -> Vec3 {
    use std::f64::consts::TAU;
    a + b.mul_element_wise((TAU * (c * t + d)).map(|x| x.cos()))
}

fn vec3_to_rgb(v: Vec3) -> image::Rgb<u8> {
    let conv = |v: f64| (v.clamp(0.0, 1.0) * 255.0) as u8;
    image::Rgb(v.map(conv).into())
}

#[derive(Debug, Copy, Clone)]
pub struct Palette {
    a: Vec3,
    b: Vec3,
    c: Vec3,
    d: Vec3,
}

impl Palette {
    pub fn pick(&self, t: f64) -> image::Rgb<u8> {
        let color = palette_vec(t, &self.a, &self.b, &self.c, &self.d);
        vec3_to_rgb(color)
    }
}

pub const RAINBOW: Palette = Palette {
    a: vec3(0.5, 0.5, 0.5),
    b: vec3(0.5, 0.5, 0.5),
    c: vec3(1.0, 1.0, 1.0),
    d: vec3(0.00, 0.33, 0.67),
};
pub const WHITES: Palette = Palette {
    a: vec3(0.5, 0.5, 0.5),
    b: vec3(0.5, 0.5, 0.5),
    c: vec3(1.0, 1.0, 1.0),
    d: vec3(0.00, 0.10, 0.20),
};
pub const ARCTIC: Palette = Palette {
    a: vec3(0.5, 0.5, 0.5),
    b: vec3(0.5, 0.5, 0.5),
    c: vec3(1.0, 1.0, 1.0),
    d: vec3(0.30, 0.20, 0.20),
};
pub const CITRUS: Palette = Palette {
    a: vec3(0.5, 0.5, 0.5),
    b: vec3(0.5, 0.5, 0.5),
    c: vec3(1.0, 1.0, 0.5),
    d: vec3(0.80, 0.90, 0.30),
};
pub const DUSK: Palette = Palette {
    a: vec3(0.5, 0.5, 0.5),
    b: vec3(0.5, 0.5, 0.5),
    c: vec3(1.0, 0.7, 0.4),
    d: vec3(0.00, 0.15, 0.20),
};
pub const PINK: Palette = Palette {
    a: vec3(0.5, 0.5, 0.5),
    b: vec3(0.5, 0.5, 0.5),
    c: vec3(2.0, 1.0, 0.0),
    d: vec3(0.50, 0.20, 0.25),
};
pub const GLOW: Palette = Palette {
    a: vec3(0.8, 0.5, 0.4),
    b: vec3(0.2, 0.4, 0.2),
    c: vec3(2.0, 1.0, 1.0),
    d: vec3(0.00, 0.25, 0.25),
};

const DEFAULT_PALETTES: [Palette; 7] = [RAINBOW, WHITES, ARCTIC, CITRUS, DUSK, PINK, GLOW];

pub struct DefaultPalettes;

impl Distribution<Palette> for DefaultPalettes {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Palette {
        *DEFAULT_PALETTES.choose(rng).unwrap()
    }
}

fn sample_vec3<R, D>(rng: &mut R, dist: D) -> Vec3
where
    R: rand::Rng + ?Sized,
    D: Distribution<f64>,
{
    vec3(dist.sample(rng), dist.sample(rng), dist.sample(rng))
}

/// This type implements a random distribution of palettes.
pub struct PhaseShiftPalette;

impl Distribution<Palette> for PhaseShiftPalette {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Palette {
        // Sample base brightness for each channel with a mode of 0.5,
        // and a maximum brightness (relative to the base).
        let base_brightness = sample_vec3(rng, Pert::new(0.0, 1.0, 0.5).unwrap());
        let max_brightness_ratio = sample_vec3(rng, Pert::new(0.0, 1.0, 0.8).unwrap());

        // Uniformly sample a random phase shift for each color channel.
        let variance: f64 = rng.gen();
        let phase = sample_vec3(rng, Uniform::new(0.0, variance));

        Palette {
            a: base_brightness,
            b: base_brightness.zip(max_brightness_ratio, |v, r| (1.0 - v) * r),
            c: vec3(1.0, 1.0, 1.0),
            d: phase,
        }
    }
}

pub struct MonotonePalette;

/// Like [PhaseShiftPalette], but the phases are centered around 0.5
/// with little variance, making the palette appear in one monotone hue.
impl Distribution<Palette> for MonotonePalette {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Palette {
        let base_brightness = sample_vec3(rng, Pert::new(0.0, 1.0, 0.5).unwrap());
        let max_brightness_ratio = sample_vec3(rng, Pert::new(0.0, 1.0, 0.8).unwrap());
        let phase = sample_vec3(rng, Pert::new_with_shape(0.0, 1.0, 0.5, 100.0).unwrap());

        Palette {
            a: base_brightness,
            b: base_brightness.zip(max_brightness_ratio, |v, r| (1.0 - v) * r),
            c: sample_vec3(rng, Uniform::new(0.5, 1.0)),
            d: phase,
        }
    }
}
