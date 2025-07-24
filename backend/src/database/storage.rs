use anyhow::{Context, Result, anyhow};
use aws_sdk_s3::Client;
use aws_sdk_s3::config::{Credentials, Region};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::{Delete, ObjectIdentifier};
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
        let bucket_name = env::var("R2_BUCKET_NAME")
            .context("Environment variable R2_BUCKET_NAME is not set")?;
        let account_id = env::var("R2_ACCOUNT_ID")
            .context("Environment variable R2_ACCOUNT_ID is not set")?;
        let access_key_id = env::var("R2_ACCESS_KEY_ID")
            .context("Environment variable R2_ACCESS_KEY_ID is not set")?;
        let secret_access_key = env::var("R2_SECRET_ACCESS_KEY")
            .context("Environment variable R2_SECRET_ACCESS_KEY is not set")?;
        let public_cdn_url = env::var("R2_PUBLIC_CDN_URL")
            .context("Environment variable R2_PUBLIC_CDN_URL is not set")?;

        // Construct the S3 endpoint URL for Cloudflare R2
        let endpoint_url =
            format!("https://{account_id}.r2.cloudflarestorage.com");

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
    /// * `key` - The full path and filename for the object in the bucket (e.g., "series-title/chapter-1/01.avif").
    /// * `data` - The raw bytes of the object to upload.
    /// * `content_type` - The MIME type of the object (e.g., "image/avif").
    /// The full public CDN URL to the uploaded object.
    pub async fn upload_image_objects(
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
            .with_context(|| {
                format!(
                    "Failed to upload object with key '{}' to R2 bucket",
                    key
                )
            })?;

        // Construct the public URL
        let public_url = format!("{}/{}", self.public_cdn_url, key);
        println!("[STORAGE] Successfully uploaded object to: {}", public_url);
        Ok(public_url)
    }

    /// Deletes multiple objects from the R2 bucket.
    /// * `keys` - A vector of object keys to delete.
    pub async fn delete_image_objects(&self, keys: Vec<String>) -> Result<()> {
        // If there are no keys to delete, do nothing.
        if keys.is_empty() {
            println!("[STORAGE] No objects to delete");
            return Ok(());
        }

        // Convert the list of key strings into a list of S3 ObjectIdentifiers
        let objects_to_delete: Result<Vec<_>, _> = keys
            .into_iter()
            .map(|key| ObjectIdentifier::builder().key(key).build())
            .collect();

        let objects_to_delete = objects_to_delete
            .context("Failed to build ObjectIdentifier objects")?;

        let delete_payload = Delete::builder()
            .set_objects(Some(objects_to_delete))
            .build()
            .map_err(|e| {
                anyhow::anyhow!("Failed to build Delete payload: {}", e)
            })?;

        // Send the delete_objects request.
        let result = self
            .client
            .delete_objects()
            .bucket(&self.bucket_name)
            .delete(delete_payload)
            .send()
            .await
            .with_context(
                || "Failed to send delete_objects request to R2 bucket",
            )?;

        let errors = result.errors();
        if !errors.is_empty() {
            // Log errors but don't fail the entire operation if some objects were deleted.
            // [NOTE] Might want to handle this more robustly depending on needs.
            eprintln!("[STORAGE] Encountered errors while deleting objects:");
            for error in errors {
                eprintln!(
                    "  - Key: {}, Code: {}, Message: {}",
                    error.key().unwrap_or("Unknown"),
                    error.code().unwrap_or("Unknown"),
                    error.message().unwrap_or("Unknown")
                );
            }
            return Err(anyhow::anyhow!(
                "Some objects failed to delete from R2."
            ));
        }

        // Get count of deleted objects
        let deleted_count =
            result.deleted.as_ref().map_or(0, |deleted| deleted.len());
        println!(
            "[STORAGE] Successfully sent request to delete {} objects.",
            deleted_count
        );
        Ok(())
    }

    pub async fn upload_cover_image_file(
        &self,
        file_bytes: Vec<u8>,
        file_name: &str,
        content_type: &str,
    ) -> Result<String> {
        let body = ByteStream::from(file_bytes);

        self.client
            .put_object()
            .bucket(&self.bucket_name)
            .key(file_name)
            .body(body)
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to upload file: {:?}", e))?;

        let public_url = format!("{}/cover/{}", self.public_cdn_url, file_name);

        Ok(public_url)
    }

    pub fn public_cdn_url(&self) -> &str {
        &self.public_cdn_url
    }
}
