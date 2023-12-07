use std::io::Cursor;

use image::ImageOutputFormat;
use rand::Rng;

mod color;
mod complex;
mod distance_estimation;
mod inverse_iteration;
mod post;

use crate::{
    complex::{Complex, JuliaParameter},
    distance_estimation::DistanceEstimation,
    inverse_iteration::InverseIteration,
};

#[derive(Debug)]
struct BoundingBox {
    min: Complex,
    max: Complex,
}

impl Default for BoundingBox {
    fn default() -> Self {
        Self {
            min: Complex::new(0.0, 0.0),
            max: Complex::new(0.0, 0.0),
        }
    }
}

#[allow(dead_code)]
impl BoundingBox {
    fn update(&mut self, p: &Complex) {
        let min = self.min;
        let max = self.max;
        self.min = Complex::new(p.re.min(min.re), p.im.min(min.im));
        self.max = Complex::new(p.re.max(max.re), p.im.max(max.im));
    }

    pub fn offset(&self, point: &Complex) -> Complex {
        point - self.min
    }

    pub fn abs_dimension(&self) -> Complex {
        self.offset(&self.max)
    }

    pub fn aspect_ratio(&self) -> f64 {
        let d = self.abs_dimension();
        d.re / d.im
    }

    pub fn width_for(&self, height: u32) -> u32 {
        let height: f64 = height.into();

        (height * self.aspect_ratio()) as u32
    }

    pub fn height_for(&self, width: u32) -> u32 {
        let width: f64 = width.into();

        (width * (1.0 / self.aspect_ratio())) as u32
    }

    fn scale(&mut self, scale: f64) {
        self.min *= scale;
        self.max *= scale;
    }

    fn to_grid(&self, point: &Complex, width: u32, height: u32) -> Option<(u32, u32)> {
        let rel: Complex = self.offset(point);
        let dim: Complex = self.abs_dimension();

        if (rel.re < 0.0 || rel.im < 0.0) || (rel.re > dim.re || rel.im > dim.im) {
            return None;
        }

        let d_x: f64 = rel.re / dim.re;
        let d_y: f64 = rel.im / dim.im;

        let x = (d_x * ((width - 1) as f64)) as u32;
        let y = (d_y * ((height - 1) as f64)) as u32;

        Some((x, y))
    }

    pub fn point_from_grid(&self, x: u32, y: u32, width: u32, height: u32) -> Complex {
        let x_rel = f64::from(x) / f64::from(width);
        let y_rel = f64::from(y) / f64::from(height);

        let dim = self.abs_dimension();

        let rel = Complex::new(dim.re * x_rel, dim.im * y_rel);
        self.min + rel
    }

    pub fn points<'b, 'p, P, Container>(
        &'b self,
        image: &'p mut image::ImageBuffer<P, Container>,
    ) -> PixelsPoints<'p, 'b, P>
    where
        P: image::Pixel,
        Container: std::ops::DerefMut<Target = [P::Subpixel]>,
    {
        PixelsPoints {
            width: image.width(),
            height: image.height(),
            pixels: image.enumerate_pixels_mut(),
            bbx: self,
        }
    }
}

impl FromIterator<Complex> for BoundingBox {
    fn from_iter<T: IntoIterator<Item = Complex>>(iter: T) -> Self {
        let mut bbx = Self::default();
        iter.into_iter().for_each(|p| bbx.update(&p));
        bbx
    }
}

struct PixelsPoints<'p, 'b, P>
where
    P: image::Pixel,
{
    width: u32,
    height: u32,
    pixels: image::buffer::EnumeratePixelsMut<'p, P>,
    bbx: &'b BoundingBox,
}

impl<'p, 'b, P> Iterator for PixelsPoints<'p, 'b, P>
where
    P: image::Pixel + 'p,
{
    type Item = (&'p mut P, Complex);

    fn next(&mut self) -> Option<Self::Item> {
        let (x, y, pixel) = self.pixels.next()?;
        let point = self.bbx.point_from_grid(x, y, self.width, self.height);
        Some((pixel, point))
    }
}

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + f64::exp(-x))
}

fn main() {
    const WIDTH: u32 = 1280;
    const ITER: usize = 10_000;

    let mut rng = rand::thread_rng();

    let c = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| rng.sample(JuliaParameter));

    println!("c = {}", c);

    let julia: InverseIteration = InverseIteration::new(c);

    let mut bbx: BoundingBox = julia // Julia::new(Complex::new(-0.12, 0.74))
        .into_iter()
        .take(ITER)
        .collect();
    bbx.scale(1.20);

    let imgbuf = {
        let mut imgbuf = image::ImageBuffer::new(WIDTH, bbx.height_for(WIDTH));
        let julia = DistanceEstimation::new(c);

        for (pixel, point) in bbx.points(&mut imgbuf) {
            let d: f64 = julia.distance(point, 1024);
            *pixel = if d <= 0.0 {
                image::Rgb([0, 0, 0])
            } else {
                let d = sigmoid((50.0 * d).sqrt());
                crate::color::WHITES.pick(d)
            };
        }

        imgbuf.save("dist_fractal.png").unwrap();

        let mut buf = Cursor::new(Vec::new());
        imgbuf.write_to(&mut buf, ImageOutputFormat::Png).unwrap();
        buf.into_inner().into_boxed_slice()
    };

    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(crate::post::post(Box::<[u8]>::leak(imgbuf)));
}
