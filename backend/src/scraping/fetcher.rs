use anyhow::{Context, Result};
use backon::{BackoffBuilder, ExponentialBuilder, Retryable};
use bytes::Bytes;
use reqwest::Client;

/// Determines whether a network error should trigger a retry attempt
/// Retry Strategy:
/// - Retry: Server errors (5xx), timeouts, connection issues, rate limits (429)
/// - Don't retry: Client errors (4xx except 429), parsing errors, other failures
///
/// We use this approach because:
/// - Server errors are often temporary (server restart, maintenance, etc.)
/// - Timeouts might succeed on retry with better network conditions
/// - Rate limits (429) usually resolve after waiting
fn is_transient_error(e: &anyhow::Error) -> bool {
    // Attempt to downcast the error to a reqwest::Error to inspect it
    if let Some(req_err) = e.downcast_ref::<reqwest::Error>() {
        // Retry if the request timed out or if there was a connection issue
        if req_err.is_timeout() || req_err.is_connect() {
            return true;
        }

        // Retry on specific status codes
        if let Some(status) = req_err.status() {
            // Retry on 5xx server errors or 429 Too Many Requests.
            return status.is_server_error() || status.as_u16() == 429;
        }
    }
    // For all other errors, we don't retry
    false
}

/// Generic fetch function that handles the core logic of sending a request
/// This is the heart of our fetching system. It handles:
/// 1. Making HTTP requests with retry logic
/// 2. Status code validation
/// 3. Exponential backoff between retries
/// 4. Flexible response processing (HTML, bytes, JSON, etc.)
///
/// Generic Parameters Explained:
/// - `T`: The final return type (String, Bytes, etc.)
/// - `F`: The processor function type
/// - `Fut`: The Future returned by the processor function
async fn fetch_with_retry<T, F, Fut>(
    client: &Client,
    url: &str,
    // This function takes a successful HTTP response and converts it to type T
    processor: F,
) -> Result<T>
where
    // F must be a function that takes Response and returns a Future<Result<T>>
    F: Fn(reqwest::Response) -> Fut,
    // Fut must be a Future that resolves to Result<T>
    Fut: Future<Output = Result<T>>,
{
    // Configure exponential backoff
    // [NOTE] Can customize it further here, `.with_max_times(5)`
    let backoff = ExponentialBuilder::default().build();

    // Define the operation we want to retry
    // This closure captures all the variables it needs (client, url, processor)
    let operation = || async {
        // This can fail due to: DNS resolution, connection refused, timeouts, etc.
        let response = client
            .get(url)
            .send()
            .await
            .with_context(|| format!("Failed to send request to {}", url))?;

        // Check if HTTP status indicates success (2xx) `Ok`
        // `error_for_status()` will convert a 4xx or 5xx status code into an `Errors`.
        // Why: HTTP request "succeeded" but server said "no" (404, 500, etc.)
        let response = response
            .error_for_status()
            .with_context(|| format!("Request to {} returned a non-success status", url))?;

        println!("[FETCHER] HTML from {} fetched successfully", url);

        processor(response).await
    };

    // Execute the operation with the retry logic, only retry on transient errors
    operation
        .retry(backoff)
        .when(|e| {
            // Decides whether to retry based on the error
            let should_retry = is_transient_error(e);
            if should_retry {
                println!(
                    "[FETCHER] Encountered transient error for {}. Retrying...",
                    url
                );
            } else {
                println!(
                    "[FETCHER] Encountered permanent error for {}. Not retrying.",
                    url
                );
            }
            should_retry
        })
        .await
}

/// PUBLIC API: Fetches HTML content from a given URL using a provided HTTP client.
/// This function wraps generic fetch_with_retry with HTML-specific processing.
pub async fn fetch_html(client: &Client, url: &str) -> Result<String> {
    println!("[FETCHER] Attempting to fetch HTML from {}", url);

    // Call generic fetch function with binary-specific processor
    fetch_with_retry(client, url, |response| async {
        // This can fail if: response is not valid UTF-8, connection drops during read
        response
            .text()
            .await
            .with_context(|| format!("Failed to read response body from {}", url))
    })
    .await
}

/// PUBLIC API: Fetch binary data (bytes) of a resource (images, files, etc.) from a URL
pub async fn fetch_image_bytes(client: &Client, url: &str) -> Result<Bytes> {
    println!("[FETCHER] Fetching image bytes from {}", url);

    // Call generic fetch function with binary-specific processor
    fetch_with_retry(client, url, |response| async {
        // This preserves the exact binary data without any text conversion
        response
            .bytes()
            .await
            .with_context(|| format!("Failed to read bytes from response of {}", url))
    })
    .await
}

// EXAMPLE: How to extend this system for new data types
//
// If you need to fetch JSON data in the future, you could add:
//
// ```
// pub async fn fetch_json<T: serde::de::DeserializeOwned>(
//     client: &Client,
//     url: &str
// ) -> Result<T> {
//     fetch_with_retry(client, url, |response| async move {
//         let text = response.text().await?;
//         serde_json::from_str(&text).map_err(Into::into)
//     }).await
// }
// ```
