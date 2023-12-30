use std::io::Cursor;

use anyhow::Context;
use image::ImageOutputFormat;
use rand::Rng;
use rayon::prelude::{ParallelBridge, ParallelIterator};

mod bounding_box;
mod color;
mod complex;
mod distance_estimation;
mod env;
mod inverse_iteration;
mod post;

use crate::{
    color::DefaultPalettes,
    complex::JuliaParameter,
    distance_estimation::DistanceEstimation,
    env::Cmdline,
    inverse_iteration::InverseIteration,
    bounding_box::BoundingBox,
};

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + f64::exp(-x))
}

fn main() -> anyhow::Result<()> {
    const WIDTH: u32 = 1280;
    const ITER: usize = 10_000;

    let mut rng = rand::thread_rng();

    let cmdline: Cmdline = argh::from_env();
    let c = cmdline
        .parameter
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
        let palette = rng.sample(DefaultPalettes);

        bbx.points(&mut imgbuf)
            .par_bridge()
            .for_each(|(pixel, point)| {
                let d: f64 = julia.distance(point, 1024);
                *pixel = if d <= 0.0 {
                    image::Rgb([0, 0, 0])
                } else {
                    let d = sigmoid((50.0 * d).sqrt());
                    palette.pick(d)
                };
            });

        imgbuf
    };

    match cmdline.action {
        env::Action::Save(save) => imgbuf
            .save(&save.path)
            .with_context(|| format!("Failed to save image to {}", save.path.display())),
        env::Action::Post(post) => {
            let mut buf = Cursor::new(Vec::new());
            imgbuf
                .write_to(&mut buf, ImageOutputFormat::Png)
                .context("Failed to encode image")?;
            let buf = buf.into_inner().into_boxed_slice();

            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(crate::post::post(
                Box::<[u8]>::leak(buf),
                format!(r"Julia set of the day: \[c = {}\]", c),
                post.status_visibility,
            ))
        }
    }
}
