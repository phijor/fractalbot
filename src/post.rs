use std::{env, time::Duration};

use futures_retry::{ErrorHandler, FutureRetry, RetryPolicy};
use megalodon::{
    entities::{Attachment, StatusVisibility, UploadMedia},
    error::{self, Error, OwnError},
    megalodon::PostStatusInputOptions,
    response::Response,
    Megalodon,
    SNS::Mastodon,
};

type Result<T> = std::result::Result<T, Error>;

pub async fn post(image_data: &'static [u8]) {
    let Ok(instance) = env::var("MASTODON_INSTANCE_URL") else {
        eprintln!("$MASTODON_INSTANCE_URL not provided");
        return;
    };

    let Ok(token) = env::var("MASTODON_ACCESS_TOKEN") else {
        eprintln!("$MASTODON_ACCESS_TOKEN not provided");
        return;
    };

    let user_agent = format!(
        "fractalbot/{} (@phijor@types.pl)",
        env!("CARGO_PKG_VERSION")
    );

    let client = megalodon::generator(Mastodon, instance, Some(token), Some(user_agent));

    let res = match client.upload_media_reader(Box::new(image_data), None).await {
        Ok(res) => res,
        Err(err) => {
            eprintln!("Failed to upload media: {err}");
            return;
        }
    };

    let Ok(media) = resolve_uploaded_media(client.as_ref(), res.json()).await else {
        eprintln!("Failed to upload media");
        return;
    };

    let _ = match client
        .post_status(
            "Hello, world".into(),
            Some(&PostStatusInputOptions {
                media_ids: Some(vec![media.id]),
                visibility: Some(StatusVisibility::Private),
                ..Default::default()
            }),
        )
        .await
    {
        Ok(res) => res,
        Err(err) => {
            eprintln!("Failed to post status: {:#?}", err);
            return;
        }
    };
}

async fn resolve_uploaded_media(
    client: &(dyn Megalodon + Send + Sync),
    upload: UploadMedia,
) -> Result<Attachment> {
    match upload {
        UploadMedia::Attachment(attachment) => Ok(attachment),
        UploadMedia::AsyncAttachment(async_attachment) => FutureRetry::new(
            || async {
                let res: Response<Attachment> =
                    client.get_media(async_attachment.id.clone()).await?;
                Ok(res.json())
            },
            RetryPartialContent::with_attempts(5),
        )
        .await
        .map(|(res, _)| res)
        .map_err(|(err, _)| err),
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
