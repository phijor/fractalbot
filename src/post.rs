use std::env;

use anyhow::{Context, Result};
use log::info;
use megalodon::{
    entities::{Attachment, StatusVisibility, UploadMedia},
    error::{self, Error, OwnError},
    megalodon::PostStatusInputOptions,
    Megalodon,
    SNS::Mastodon,
};

use crate::retry::{retry, Retry};

struct Status {
    client: Box<dyn Megalodon + Send + Sync>,
}

impl Status {
    async fn upload_image(&self, image_data: &'static [u8]) -> Result<UploadMedia> {
        retry(Retry::any(), || async {
            self.client
                .upload_media_reader(Box::new(image_data), None)
                .await
                .map(|res| res.json())
        })
        .await
    }

    async fn resolve_uploaded_media(&self, upload: UploadMedia) -> Result<Attachment> {
        match upload {
            UploadMedia::Attachment(attachment) => Ok(attachment),
            UploadMedia::AsyncAttachment(async_attachment) => {
                let retry_partian_content = Retry::when(|err: &Error| {
                    matches!(
                        err,
                        Error::OwnError(OwnError {
                            kind: error::Kind::HTTPPartialContentError,
                            ..
                        })
                    )
                });
                retry(retry_partian_content, || async {
                    self.client
                        .get_media(async_attachment.id.clone())
                        .await
                        .map(|res| res.json())
                })
                .await
            }
        }
    }

    async fn post(
        &self,
        media_id: String,
        description: String,
        visibility: StatusVisibility,
    ) -> Result<()> {
        retry(Retry::any(), || async {
            self.client
                .post_status(
                    description.clone(),
                    Some(&PostStatusInputOptions {
                        media_ids: Some(vec![media_id.clone()]),
                        visibility: Some(visibility.clone()),
                        ..Default::default()
                    }),
                )
                .await
                .map(drop)
        })
        .await
    }
}

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

    let status = Status { client };

    info!("Uploading image...");
    let media = status
        .upload_image(image_data)
        .await
        .context("Failed to upload image")?;

    info!("Resolving uploaded image...");
    let media = status
        .resolve_uploaded_media(media)
        .await
        .context("Failed to resolve uploaded image")?;

    info!("Uploaded image has ID {}", media.id);

    info!("Posting status...");
    status
        .post(media.id, description, visibility)
        .await
        .context("Failed to post status")
}
