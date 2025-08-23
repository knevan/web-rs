use anyhow::{Context, Result};
use rand::Rng;
use std::time::Duration;
use tokio::time::sleep;
use url::Url;

// Converts a relative URL string to an absolute URL string, given a base URL.
pub fn to_absolute_url(
    base_url_str: &str,
    relative_url_str: &str,
) -> Result<String> {
    let base_url = Url::parse(base_url_str)
        .with_context(|| format!("Base URL not valid: {}", base_url_str))?;

    let absolute_url = base_url.join(relative_url_str).with_context(|| {
        format!(
            "Failed to append URL: {} with {}",
            base_url_str, relative_url_str
        )
    })?;

    Ok(absolute_url.into()) // .into() is equivalent to .to_string() for Url
}

/// Pauses execution asynchronously for a random duration between `min_secs` and `max_secs`.
/// If `min_secs` is greater than or equal to `max_secs`, it sleeps for `min_secs`.
pub async fn random_sleep_time(min_secs: u64, max_secs: u64) {
    let sleep_duration_seconds = if min_secs >= max_secs {
        min_secs
    } else {
        rand::rng().random_range(min_secs..=max_secs)
    };

    // If duration 0 don't sleep, otherwise sleep for a random duration
    if sleep_duration_seconds > 0 {
        sleep(Duration::from_secs(sleep_duration_seconds)).await;
    }
}
