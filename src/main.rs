use std::io::Cursor;
use std::ops::Deref;

use anyhow::{Context, Result};
use humansize::SizeFormatter;
use image::{ImageBuffer, ImageFormat};
use indoc::formatdoc;
use log::{debug, info};
use rand::Rng;
use rayon::prelude::{ParallelBridge, ParallelIterator};

mod bounding_box;
mod color;
mod complex;
mod distance_estimation;
mod env;
mod inverse_iteration;

use crate::{
    bounding_box::BoundingBox,
    color::MonotonePalette,
    complex::Complex,
    distance_estimation::{DistanceEstimation, MandelbrotBoundary},
    env::{Cmdline, Environment, Post},
    inverse_iteration::InverseIteration,
};

/// Squeeze values in range [0, infty) into [0, 1).
#[inline]
fn squeeze(x: f64) -> f64 {
    f64::exp(-x)
}

fn logger_init() {
    use env_logger::{Builder, Env};

    let style = if std::env::var_os("NO_COLOR").is_some() {
        "never"
    } else {
        "auto"
    };

    let env = Env::new()
        .filter_or("FRACTALBOT_LOG", "info")
        .write_style_or("FRACTALBOT_LOG_STYLE", style);

    Builder::from_env(env).init()
}

fn main() -> anyhow::Result<()> {
    logger_init();

    const WIDTH: u32 = 1280;
    const HEIGHT: u32 = 1280;
    const ITER: usize = 10_000;
    const MAX_ITER: usize = 4096;

    let mut rng = rand::thread_rng();

    let cmdline: Cmdline = argh::from_env();
    let c = cmdline
        .parameter
        .unwrap_or_else(|| rng.sample(MandelbrotBoundary { max_iter: MAX_ITER }));

    info!("Julia parameter: c = {c}");

    let julia: InverseIteration = InverseIteration::new(c);

    let mut bbx: BoundingBox = julia // Julia::new(Complex::new(-0.12, 0.74))
        .into_iter()
        .take(ITER)
        .collect();
    bbx.scale(1.20);

    info!(
        "Bounding box has aspect ratio of {ratio:.2}:1",
        ratio = bbx.aspect_ratio()
    );

    let imgbuf = {
        let (width, height) = bbx.fit(WIDTH * 2, HEIGHT * 2);
        let mut imgbuf = image::ImageBuffer::new(width, height);
        let julia = DistanceEstimation::new(c, MAX_ITER);

        let palette = rng.sample(MonotonePalette);
        let sharpness = if julia.is_connected() {
            info!("Julia set is connected");
            25.0
        } else {
            info!("Julia set is disconnected");
            100.0
        };

        debug!("Palette: {:.2?}", palette);
        debug!("Color for d=0.0: {:?}", palette.pick(0.0));

        let set_color = move |(pixel, point): (&mut _, Complex)| {
            let d: f64 = julia.distance(point);
            *pixel = if d <= 0.0 {
                image::Rgb([0, 0, 0])
            } else {
                let d = squeeze((sharpness * d).sqrt());
                palette.pick(d)
            };
        };

        bbx.points(&mut imgbuf).par_bridge().for_each(set_color);

        info!("Downscaling supersampled image...");
        image::imageops::resize(
            &imgbuf,
            width / 2,
            height / 2,
            image::imageops::FilterType::Triangle,
        )
    };

    match cmdline.action {
        env::Action::Save(save) => {
            info!("Saving image to {}", save.path.display());
            imgbuf
                .save(&save.path)
                .with_context(|| format!("Failed to save image to {}", save.path.display()))
        }
        env::Action::Post(Post { status_visibility }) => {
            let description = formatdoc! {r#"
                Julia set of the day:
                \[
                    c = {c}
                \]

                #fractal #generative
            "#};

            post_status(imgbuf, description, status_visibility)
        }
    }
}

fn encode_png<Container>(imgbuf: ImageBuffer<image::Rgb<u8>, Container>) -> Result<&'static [u8]>
where
    Container: Deref<Target = [u8]>,
{
    let mut encode_buffer = Cursor::new(Vec::new());
    imgbuf
        .write_to(&mut encode_buffer, ImageFormat::Png)
        .context("Failed to encode image")?;
    let buf = encode_buffer.into_inner().into_boxed_slice();

    Ok(Box::leak(buf))
}

fn post_status<Container>(
    imgbuf: ImageBuffer<image::Rgb<u8>, Container>,
    description: String,
    visibility: fractalbot_post::StatusVisibility,
) -> Result<()>
where
    Container: Deref<Target = [u8]>,
{
    info!("Encoding image");
    let encoded_image = encode_png(imgbuf)?;

    info!(
        "Posting image to fediverse (size: {})",
        SizeFormatter::new(encoded_image.len(), humansize::DECIMAL)
    );

    let user_agent = format!(
        "fractalbot/{} (@phijor@types.pl)",
        env!("CARGO_PKG_VERSION")
    );
    let env = Environment::from_env()?;
    let client = fractalbot_post::Client::new(env.instance_url, env.access_token, user_agent);

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(client.post_status_with_image(encoded_image, description, visibility))
        .context("Failed to post image")
}
