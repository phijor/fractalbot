use anyhow::{Context, Result};
use argh::FromArgs;
use megalodon::entities::StatusVisibility;

use std::env;

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
/// Post a random fractal to the fediverse
pub struct Cmdline {
    #[argh(switch, short = 'n', long = "dry-run")]
    /// do not send post
    pub dry_run: bool,

    #[argh(option, default = "StatusVisibility::Private")]
    /// visibility of the status (public, unlisted, private or direct)
    pub status_visibility: StatusVisibility,

    #[argh(positional)]
    pub parameter: Option<Complex>,
}
