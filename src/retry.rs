use std::time::Duration;

use anyhow::{Context, Result};
use futures::TryFuture;
use futures_retry::{ErrorHandler, FutureFactory, FutureRetry, RetryPolicy};
use log::{info, warn};
use megalodon::error::Error;

pub struct Retry<F> {
    attempts: usize,
    should_retry: F,
}

impl<F> Retry<F> {
    pub fn when(should_retry: F) -> Self {
        Self {
            attempts: 5,
            should_retry,
        }
    }
}

impl Retry<fn(&Error) -> bool> {
    pub fn any() -> Self {
        Retry::when(|_| true)
    }
}

fn annotate_retries<T, E>(res: std::result::Result<(T, usize), (E, usize)>) -> Result<T>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn retries_str(num_retries: usize) -> &'static str {
        if num_retries == 1 {
            "retry"
        } else {
            "retries"
        }
    }

    match res {
        Ok((res, num_retries)) => {
            if num_retries > 1 {
                info!("Succeeded after {num_retries} retries");
            }
            Ok(res)
        }
        Err((err, num_retries)) => Err(err).context({
            let retry = retries_str(num_retries);
            format!("Failed to upload image after {num_retries} {retry}",)
        }),
    }
}

pub async fn retry<E, F>(
    error_action: Retry<E>,
    factory: F,
) -> Result<<F::FutureItem as TryFuture>::Ok>
where
    F: FutureFactory,
    Retry<E>: ErrorHandler<<F::FutureItem as TryFuture>::Error>,
    // Ahh yes, exactly.
    <Retry<E> as ErrorHandler<<<F as FutureFactory>::FutureItem as TryFuture>::Error>>::OutError:
        std::error::Error + Send + Sync + 'static,
{
    let res = FutureRetry::new(factory, error_action).await;
    annotate_retries(res)
}

impl<F> ErrorHandler<Error> for Retry<F>
where
    F: Fn(&Error) -> bool,
{
    type OutError = Error;

    fn handle(&mut self, attempt: usize, err: Error) -> RetryPolicy<Self::OutError> {
        if attempt > self.attempts {
            return RetryPolicy::ForwardError(err);
        }

        if (self.should_retry)(&err) {
            let delay = Duration::from_secs(2u64.pow(attempt as u32));
            warn!(
                "Action failed on attempt {attempt}/{max_attempts}, waiting for {delay_sec:.1}s (Error: {err})",
                max_attempts = self.attempts,
                delay_sec = delay.as_secs_f32()
            );
            RetryPolicy::WaitRetry(delay)
        } else {
            RetryPolicy::ForwardError(err)
        }
    }
}
