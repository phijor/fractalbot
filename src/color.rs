use cgmath::{prelude::*, vec3, Vector3};
use rand::{distributions::Distribution, seq::SliceRandom};

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
