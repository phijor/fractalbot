use anyhow::{Context, Result};
use argh::FromArgs;
use fractalbot_post::StatusVisibility;

use std::env;
use std::path::PathBuf;

use crate::complex::Complex;

#[derive(Debug)]
pub struct Environment {
    pub instance_url: String,
    pub access_token: String,
}

impl Environment {
    pub fn from_env() -> Result<Self> {
        let instance_url =
            env::var("MASTODON_INSTANCE_URL").context("MASTODON_INSTANCE_URL not set")?;
        let access_token =
            env::var("MASTODON_ACCESS_TOKEN").context("MASTODON_ACCESS_TOKEN not set")?;

        Ok(Self {
            instance_url,
            access_token,
        })
    }
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum Action {
    Save(Save),
    Post(Post),
}

#[derive(FromArgs, PartialEq, Debug)]
/// Save the image to disk.
#[argh(subcommand, name = "save")]
pub struct Save {
    #[argh(positional, default = r#""fractal.png".into()"#)]
    /// path to the image on disk
    pub path: PathBuf,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Post the image to the fediverse.
#[argh(subcommand, name = "post")]
pub struct Post {
    #[argh(option, default = "StatusVisibility::Private")]
    /// visibility of the status (public, unlisted, private or direct)
    pub status_visibility: StatusVisibility,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Genarate a random fractal and share it.
pub struct Cmdline {
    #[argh(option, short = 'c', long = "julia-parameter")]
    /// complex number c parametrizing the generating polynomial f(z) = z² + c
    pub parameter: Option<Complex>,

    #[argh(subcommand)]
    pub action: Action,
}
