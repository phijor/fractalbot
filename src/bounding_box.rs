use crate::complex::Complex;

#[derive(Debug)]
pub struct BoundingBox {
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

    pub fn fit(&self, width: u32, height: u32) -> (u32, u32) {
        if self.aspect_ratio() > 1.0 {
            // Bounding box is wider than tall
            (width, self.height_for(width))
        } else {
            (self.width_for(height), height)
        }
    }

    pub fn scale(&mut self, scale: f64) {
        self.min *= scale;
        self.max *= scale;
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

pub struct PixelsPoints<'p, 'b, P>
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
