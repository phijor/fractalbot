use std::sync::Arc;

use anyhow::{Context, Result};
use log::info;
use megalodon::{
    entities::{Attachment, UploadMedia},
    error::{self, Error, OwnError},
    megalodon::PostStatusInputOptions,
    Megalodon,
    SNS::Mastodon,
};

pub use megalodon::entities::StatusVisibility;

use crate::retry::{retry, Retry};

pub struct Client {
    pub client: Arc<dyn Megalodon + Send + Sync>,
}

impl Client {
    pub fn new(instance_url: String, access_token: String, user_agent: String) -> Self {
        let client: Arc<_> =
            megalodon::generator(Mastodon, instance_url, Some(access_token), Some(user_agent))
                .into();
        Self { client }
    }

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

    pub async fn post_status(
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

    pub async fn post_status_with_image(
        &self,
        image_data: &'static [u8],
        description: String,
        visibility: StatusVisibility,
    ) -> Result<()> {
        info!("Uploading image...");
        let media = self
            .upload_image(image_data)
            .await
            .context("Failed to upload image")?;

        info!("Resolving uploaded image...");
        let media = self
            .resolve_uploaded_media(media)
            .await
            .context("Failed to resolve uploaded image")?;

        info!("Uploaded image has ID {}", media.id);

        info!("Posting status...");
        self.post_status(media.id, description, visibility)
            .await
            .context("Failed to post status")
    }
}
