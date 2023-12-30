use std::{env, time::Duration};

use anyhow::{Context, Result};
use futures_retry::{ErrorHandler, FutureRetry, RetryPolicy};
use log::info;
use megalodon::{
    entities::{Attachment, StatusVisibility, UploadMedia},
    error::{self, Error, OwnError},
    megalodon::PostStatusInputOptions,
    response::Response,
    Megalodon,
    SNS::Mastodon,
};

pub async fn post(
    image_data: &'static [u8],
    description: String,
    visibility: StatusVisibility,
) -> Result<()> {
    let env = crate::env::Environment::from_env()?;

    let user_agent = format!(
        "fractalbot/{} (@phijor@types.pl)",
        env!("CARGO_PKG_VERSION")
    );

    let client = megalodon::generator(
        Mastodon,
        env.instance_url,
        Some(env.access_token),
        Some(user_agent),
    );

    info!("Uploading image");
    let res = client
        .upload_media_reader(Box::new(image_data), None)
        .await
        .context("Failed to upload image")?;

    info!("Resolving uploaded image");
    let media = resolve_uploaded_media(client.as_ref(), res.json())
        .await
        .context("Failed to resolve uploaded image")?;
    info!("Uploaded image has ID {}", media.id);

    client
        .post_status(
            description,
            Some(&PostStatusInputOptions {
                media_ids: Some(vec![media.id]),
                visibility: Some(visibility),
                ..Default::default()
            }),
        )
        .await
        .context("Failed to post status")?;

    Ok(())
}

async fn resolve_uploaded_media(
    client: &(dyn Megalodon + Send + Sync),
    upload: UploadMedia,
) -> std::result::Result<Attachment, megalodon::error::Error> {
    match upload {
        UploadMedia::Attachment(attachment) => Ok(attachment),
        UploadMedia::AsyncAttachment(async_attachment) => {
            let res = FutureRetry::new(
                || async {
                    let res: Response<Attachment> =
                        client.get_media(async_attachment.id.clone()).await?;
                    std::result::Result::Ok(res.json())
                },
                RetryPartialContent::with_attempts(5),
            )
            .await;

            match res {
                Ok((attachment, _tries)) => Ok(attachment),
                Err((err, _tries)) => Err(err),
            }
        }
    }
}

struct RetryPartialContent {
    attempts: usize,
}

impl RetryPartialContent {
    fn with_attempts(attempts: usize) -> Self {
        Self { attempts }
    }
}

impl ErrorHandler<Error> for RetryPartialContent {
    type OutError = Error;

    fn handle(&mut self, attempt: usize, err: Error) -> RetryPolicy<Self::OutError> {
        if attempt > self.attempts {
            return RetryPolicy::ForwardError(err);
        }

        match err {
            Error::OwnError(OwnError {
                kind: error::Kind::HTTPPartialContentError,
                ..
            }) => RetryPolicy::WaitRetry(Duration::from_secs(2u64.pow(attempt as u32))),
            err => RetryPolicy::ForwardError(err),
        }
    }
}
