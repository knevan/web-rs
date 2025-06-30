use anyhow::Result;
use url::Url;

use crate::db::db::MangaSeries;

/// Utility function to get the host from a series's source URL.
pub fn get_series_host(series: &MangaSeries) -> Result<String> {
    let series_main_page_url = series.current_source_url.as_ref().ok_or_else(|| {
        anyhow::anyhow!("Series source URL does not exist for '{}'", series.title)
    })?;

    let series_host = Url::parse(series_main_page_url)?
        .host_str()
        .ok_or_else(|| anyhow::anyhow!("Could not get host from URL: {}", series_main_page_url))?
        .to_string();

    Ok(series_host)
}
