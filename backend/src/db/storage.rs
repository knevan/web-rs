use anyhow::{Context, Result};
use aws_sdk_s3::Client;
use aws_sdk_s3::config::{Credentials, Region};
use aws_sdk_s3::primitives::ByteStream;
use std::env;

/// A client for interacting with an S3-compatible object storage like Cloudflare R2.
#[derive(Clone)]
pub struct StorageClient {
    client: Client,
    bucket_name: String,
    public_cdn_url: String,
}

impl StorageClient {
    /// Creates a new StorageClient from environment variables.
    /// Required environment variables:
    /// - `R2_BUCKET_NAME`: The name of your R2 bucket.
    /// - `R2_ACCOUNT_ID`: Your Cloudflare account ID.
    /// - `R2_ACCESS_KEY_ID`: Your R2 access key ID.
    /// - `R2_SECRET_ACCESS_KEY`: Your R2 secret access key.
    /// - `R2_PUBLIC_CDN_URL`: The public URL of your bucket (e.g., https://pub-xxxxxxxx.r2.dev or your custom domain).
    pub async fn new_from_env() -> Result<Self> {
        let bucket_name =
            env::var("R2_BUCKET_NAME").context("Environment variable R2_BUCKET_NAME is not set")?;
        let account_id =
            env::var("R2_ACCOUNT_ID").context("Environment variable R2_ACCOUNT_ID is not set")?;
        let access_key_id = env::var("R2_ACCESS_KEY_ID")
            .context("Environment variable R2_ACCESS_KEY_ID is not set")?;
        let secret_access_key = env::var("R2_SECRET_ACCESS_KEY")
            .context("Environment variable R2_SECRET_ACCESS_KEY is not set")?;
        let public_cdn_url = env::var("R2_PUBLIC_CDN_URL")
            .context("Environment variable R2_PUBLIC_CDN_URL is not set")?;

        // Construct the S3 endpoint URL for Cloudflare R2
        let endpoint_url = format!("https://{}.r2.cloudflarestorage.com", account_id);

        // Create a static credentials provider
        let credentials = Credentials::new(
            access_key_id,
            secret_access_key,
            None,                            // session_token
            None,                            // expiry
            "cloudflare-r2-static-provider", // provider_name
        );

        // Load AWS configuration, overriding the endpoint and credentials
        let config = aws_config::from_env()
            .endpoint_url(endpoint_url)
            .credentials_provider(credentials)
            // A region is often required, even for R2. 'auto' is a safe default
            .region(Region::new("auto"))
            .load()
            .await;

        let client = Client::new(&config);

        println!(
            "[STORAGE] R2 Storage client initialized for bucket: {}",
            bucket_name
        );

        Ok(Self {
            client,
            bucket_name,
            public_cdn_url,
        })
    }

    /// Uploads an object (an image) to the R2 bucket.
    ///
    /// # Arguments
    /// * `key` - The full path and filename for the object in the bucket (e.g., "series-title/chapter-1/01.avif").
    /// * `data` - The raw bytes of the object to upload.
    /// * `content_type` - The MIME type of the object (e.g., "image/avif").
    ///
    /// # Returns
    /// The full public CDN URL to the uploaded object.
    pub async fn upload_object(
        &self,
        key: &str,
        data: Vec<u8>,
        content_type: &str,
    ) -> Result<String> {
        let byte_stream = ByteStream::from(data);

        self.client
            .put_object()
            .bucket(&self.bucket_name)
            .key(key)
            .body(byte_stream)
            .content_type(content_type)
            .send()
            .await
            .with_context(|| format!("Failed to upload object with key '{}' to R2 bucket", key))?;

        // Construct the public URL
        let public_url = format!("{}/{}", self.public_cdn_url, key);
        println!("[STORAGE] Successfully uploaded object to: {}", public_url);
        Ok(public_url)
    }
}
