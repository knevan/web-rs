use anyhow::{Result as AnyhowResult, anyhow};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rand::Rng;
use rusqlite::{
    Connection, Error as RusqliteError, OptionalExtension, Result as RusqliteResult, Row, params,
};
use std::fs;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use url::Url;

// Type alias for db connection pool
pub type DbPool = Pool<SqliteConnectionManager>;

/// Struct pepresents a Manhwa series stored in the database.
#[derive(Debug)]
pub struct MangaSeries {
    pub id: i32,
    pub title: String,
    pub description: Option<String>,
    pub cover_image_url: Option<String>,
    pub current_source_url: Option<String>,
    pub source_website_host: Option<String>,
    pub last_chapter_found_in_storage: Option<f32>, // e.g., 10.0, 10.5
    pub processing_status: String, // e.g., "pending", "monitoring", "error", "completed"
    pub check_interval_minutes: i32,
    pub last_checked_at: Option<i64>,
    pub next_checked_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Struct represent chapter
#[derive(Debug)]
pub struct Chapter {
    pub id: i32,
    pub series_id: i32,
    pub chapter_number: f32,
    pub title: Option<String>,
    pub source_url: String,
    pub created_at: i64,
}

/// Returns the current Unix timestamp in seconds.
pub fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("SystemTime before UNIX EPOCH! This should not happen.")
        .as_secs() as i64
}

// A helper function to extract a hostname from an optional URL string.
// This is created to avoid code duplication, following the DRY principle.
fn get_host_from_url(url_option: Option<&str>) -> Option<String> {
    url_option.and_then(|url_str| {
        Url::parse(url_str)
            .ok()
            .and_then(|url| url.host_str().map(String::from))
    })
}

// Function to initialize the database connection pool
pub fn create_db_pool(database_path: &str) -> anyhow::Result<DbPool> {
    let manager = SqliteConnectionManager::file(database_path).with_init(|conn| {
        // WAL mode for better concurrent access
        //conn.execute("PRAGMA journal_mode = WAL", [])?;

        // Optimize for concurrent reads/writes
        //conn.execute("PRAGMA synchronous = NORMAL", [])?;
        //conn.execute("PRAGMA cache_size = 2000", [])?;
        //conn.execute("PRAGMA temp_store = memory", [])?;
        //conn.execute("PRAGMA mmap_size = 268435456", [])?; // 256MB

        // Foreign key constraints
        conn.execute("PRAGMA foreign_keys = ON", [])?;

        // Busy timeout for concurrent access
        conn.execute("PRAGMA busy_timeout = 5000", [])?; // 5 seconds

        Ok(())
    });

    let pool = Pool::builder()
        .max_size(5)
        .min_idle(Some(2))
        .connection_timeout(Duration::from_secs(30))
        .idle_timeout(Some(Duration::from_secs(300)))
        .max_lifetime(Some(Duration::from_secs(1800)))
        .build(manager)?;

    Ok(pool)
}

/// Initializes the database schema by executing SQL commands from a specified file.
pub fn initialize_schema(conn: &Connection, db_sql_file_path: &str) -> AnyhowResult<()> {
    let schema_sql = fs::read_to_string(db_sql_file_path)?;
    conn.execute_batch(&schema_sql)?; // Executes one or more SQL statements
    println!(
        "[DB] Schema database from {} initialized successfully",
        db_sql_file_path
    );
    Ok(())
}

/// Database operations with connection pool
pub struct DatabaseService {
    pool: DbPool,
}

impl DatabaseService {
    pub fn new(pool: DbPool) -> DatabaseService {
        DatabaseService { pool }
    }

    /// Helper function to map a database row to a `ManhwaSeries` struct.
    fn row_manga_series(row: &Row) -> RusqliteResult<MangaSeries> {
        Ok(MangaSeries {
            id: row.get(0)?,
            title: row.get(1)?,
            description: row.get(2)?,
            cover_image_url: row.get(3)?,
            current_source_url: row.get(4)?,
            source_website_host: row.get(5)?,
            last_chapter_found_in_storage: row.get(6)?,
            processing_status: row.get(7)?,
            check_interval_minutes: row.get(8)?,
            last_checked_at: row.get(9)?,
            next_checked_at: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    }

    /// Adds a new manhwa series to the database from manual admin input.
    /// The `next_checked_at` is initially set to the current time to allow immediate processing.
    pub async fn add_manga_series(
        &self,
        title: &str,
        description: Option<&str>,
        cover_image_url: Option<&str>,
        source_url: Option<&str>,
        interval_minutes: i32,
    ) -> AnyhowResult<i64> {
        let pool = self.pool.clone();
        let title = title.to_string();
        let description = description.map(|s| s.to_string());
        let cover_image_url = cover_image_url.map(|s| s.to_string());
        let source_url = source_url.map(|s| s.to_string());

        tokio::task::spawn_blocking(move || {
            let conn = pool.get()?;
            let now = current_timestamp();
            let host = get_host_from_url(source_url.as_deref());

            conn.execute(
                "INSERT INTO manhwa_series (title, description, cover_image_url, current_source_url, source_website_host, check_interval_minutes, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    title,
                    description,
                    cover_image_url,
                    source_url,
                    host,
                    interval_minutes,
                    now, // created_at
                    now  // updated_at
                ],
            )?;

            Ok(conn.last_insert_rowid())
        })
            .await?
    }

    /// Updates an existing manhwa series chapter data
    /// if there is any broken chapter
    pub async fn update_manga_series(
        &self,
        series_id: i32,
        title: &str,
        description: Option<&str>,
        cover_image_url: Option<&str>,
        source_url: Option<&str>,
        interval_minutes: i32,
    ) -> AnyhowResult<usize> {
        let pool = self.pool.clone();
        let title = title.to_string();
        let description = description.map(|s| s.to_string());
        let cover_image_url = cover_image_url.map(|s| s.to_string());
        let source_url = source_url.map(|s| s.to_string());

        tokio::task::spawn_blocking(move || {
            let conn = pool.get()?;
            let now = current_timestamp();
            let host = get_host_from_url(source_url.as_deref());

            let rows_affected = conn.execute(
                "UPDATE manhwa_series
                    SET title = ?1, description = ?2, cover_image_url = ?3, current_source_url = ?4,
                        source_website_host = ?5, check_interval_minutes = ?6, updated_at = ?7
                 WHERE id = ?8",
                params![
                    title,
                    description,
                    cover_image_url,
                    source_url,
                    host,
                    interval_minutes,
                    now,
                    series_id
                ],
            )?;
            Ok(rows_affected)
        })
        .await?
    }

    /// Adds a new chapter to the database and returns its new ID.
    /// This function assumes the chapter does not already exist (checked by source_url uniqueness).
    pub async fn add_chapter(
        &self,
        series_id: i32,
        chapter_number: f32,
        title: Option<&str>,
        source_url: &str,
    ) -> AnyhowResult<i64> {
        let pool = self.pool.clone();
        let title = title.map(|s| s.to_string());
        let source_url = source_url.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = pool.get()?;
            conn.execute(
                "INSERT OR IGNORE INTO chapters (series_id, chapter_number, title, source_url, created_at)
     VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
        series_id,
        chapter_number,
        title,
        source_url,
        current_timestamp()
    ],
            )?;

            // If INSERT was successful, last_insert_rowid() will be the new ID.
            // If it was IGNORED, it will be 0. We then need to fetch the existing ID.
            let new_id = conn.last_insert_rowid();
            if new_id > 0 {
                Ok(new_id)
            } else {
                // The chapter already existed, so we need to find its ID.
                let existing_id = conn.query_row(
                    "SELECT id FROM chapters WHERE source_url = ?1",
                    params![source_url],
                    |row| row.get(0),
                )?;

                Ok(existing_id)
            }
        })
            .await?
    }

    /// Adds a new image entry associated with a chapter
    pub async fn add_chapter_image(
        &self,
        chapter_id: i32,
        image_order: i32,
        image_url: &str, // This will be the R2/CDN Url
    ) -> AnyhowResult<i64> {
        let pool = self.pool.clone();
        let image_url = image_url.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = pool.get()?;
            conn.execute(
                "INSERT INTO chapter_images (chapter_id, image_order, image_url, created_at) VALUES (?1, ?2, ?3, ?4)",
                params![
            chapter_id,
            image_order,
            image_url,
            current_timestamp()
        ],
            )?;
            Ok(conn.last_insert_rowid())
        })
            .await?
    }

    /// Retrieves a single manhwa series by its ID.
    pub async fn get_manhwa_series_by_id(&self, id: i32) -> AnyhowResult<Option<MangaSeries>> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let conn = pool.get()?;
            let series = conn.query_row(
                "SELECT id, title, description, cover_image_url, current_source_url, source_website_host, last_chapter_found_in_storage, processing_status, check_interval_minutes, last_checked_at, next_checked_at, created_at, updated_at FROM manhwa_series WHERE id = ?1",
                params![id],
                Self::row_manga_series,
            ).optional()?; // Handles cases where no row is found
            Ok(series)
        })
            .await?
    }

    /// Retrieves a single manhwa series by its title.
    pub async fn get_manhwa_series_by_title(
        &self,
        title: &str,
    ) -> AnyhowResult<Option<MangaSeries>> {
        let pool = self.pool.clone();
        let title = title.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = pool.get()?;
            let series = conn.query_row(
                "SELECT id, title, description, cover_image_url, current_source_url, source_website_host, last_chapter_found_in_storage, processing_status, check_interval_minutes, last_checked_at, next_checked_at, created_at, updated_at FROM manhwa_series WHERE title = ?1",
                params![title],
                Self::row_manga_series,
            ).optional()?;
            Ok(series)
        })
            .await?
    }

    /// Updates the `last_chapter_found_in_storage` for a series.
    pub async fn update_series_last_chapter_found_in_storage(
        &self,
        series_id: i32,
        chapter_number: Option<f32>,
    ) -> AnyhowResult<()> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let conn = pool.get()?;
            conn.execute(
                "UPDATE manhwa_series SET last_chapter_found_in_storage = ?1, updated_at = ?2 WHERE id = ?3",
                params![chapter_number, current_timestamp(), series_id],
            )?;
            Ok(())
        })
            .await?
    }

    /// NEWLY MOVED: Updates the source URL and source website host for a given series.
    pub async fn update_series_source_urls(
        &self,
        series_id: i32,
        new_source_url: &str,
    ) -> AnyhowResult<usize> {
        let pool = self.pool.clone();
        let new_source_url = new_source_url.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = pool.get()?;
            let now = current_timestamp();
            let new_host = Url::parse(&new_source_url)
                .ok()
                .and_then(|url| url.host_str().map(String::from));
            let rows_affected = conn.execute(
                "UPDATE manhwa_series SET current_source_url = ?1, source_website_host = ?2, updated_at = ?3 WHERE id = ?4",
                params![new_source_url, new_host, now, series_id],
            )?;
            Ok(rows_affected)
        })
            .await?
    }

    /// Update the description of a manhwa series.
    pub async fn update_series_description(
        &self,
        series_id: i32,
        description: &str,
    ) -> AnyhowResult<usize> {
        let pool = self.pool.clone();
        let description = description.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = pool.get()?;
            let rows_affected = conn.execute(
                "UPDATE manhwa_series SET description = ?1, updated_at = ?2 WHERE id = ?3",
                params![description, current_timestamp(), series_id],
            )?;
            Ok(rows_affected)
        })
        .await?
    }

    /// Updates only the processing status of a series.
    /// Useful for marking a series as "scraping" or "error" without touching check schedules.
    pub async fn update_series_processing_status(
        &self,
        series_id: i32,
        new_status: &str,
    ) -> AnyhowResult<usize> {
        let pool = self.pool.clone();
        let new_status = new_status.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = pool.get()?;
            let rows_affected = conn.execute(
                "UPDATE manhwa_series SET processing_status = ?1, updated_at = ?2 WHERE id = ?3",
                params![new_status, current_timestamp(), series_id],
            )?;
            Ok(rows_affected)
        })
        .await?
    }

    /// Updates the processing status and check schedule for a series.
    /// Typically called after a series has been processed (downloaded/parsed).
    /// If `new_next_checked_at` is `None`, it's calculated based on `check_interval_minutes`.
    pub async fn update_series_check_schedule(
        &self,
        series_id: i32,
        new_status: Option<&str>,
        new_last_checked_at: Option<i64>,
        new_next_checked_at: Option<i64>,
    ) -> AnyhowResult<usize> {
        // First, get the series data asynchronously.
        let series = self
            .get_manhwa_series_by_id(series_id)
            .await?
            .ok_or_else(|| {
                anyhow!(
                    "Failed to update schedule: Series with id {} not found.",
                    series_id
                )
            })?;

        let pool = self.pool.clone();
        let new_status = new_status.map(|s| s.to_string());

        tokio::task::spawn_blocking(move || {
            let conn = pool.get()?;
            let now = current_timestamp();

            let final_status = new_status.as_deref().unwrap_or(&series.processing_status);
            let final_last_checked_at = new_last_checked_at.unwrap_or(now);

            // Calculate next check time if not explicitly provided
            let final_next_checked_at = new_next_checked_at.unwrap_or_else(|| {
                let mut rng = rand::rng();
                let base_interval = series.check_interval_minutes as i64;
                let random_jitter = rng.random_range(-30..=30);
                let actual_interval = (base_interval + random_jitter).max(30); // Ensure minimum interval
                final_last_checked_at + (actual_interval * 60)
            });

            let rows_affected = conn.execute(
                "UPDATE manhwa_series SET processing_status = ?1, last_checked_at = ?2, next_checked_at = ?3, updated_at = ?4 WHERE id = ?5",
                params![final_status, final_last_checked_at, final_next_checked_at, now, series_id],
            )?;
            Ok(rows_affected)
        })
            .await?
    }
}

// Retrieves all manhwa series from the database, ordered by title.
// (This function was commented out in the original, uncommented and kept for completeness)
/*pub fn get_all_manhwa_series(conn: &Connection) -> Result<Vec<ManhwaSeries>> {
    let mut stmt = conn.prepare("SELECT id, title, current_source_url, source_website_host, last_chapter_found_in_storage, processing_status, check_interval_minutes, last_checked_at, next_checked_at, created_at, updated_at FROM manhwa_series ORDER BY title ASC")?;
    let series_iter = stmt.query_map([], row_manhwa_series)?;

    let mut series_list = Vec::new();
    for series in series_iter {
        series_list.push(series?); // Propagate errors from row mapping
    }
    Ok(series_list)
}*/

// Retrieves manhwa series that are due for checking.
// (This function was commented out in the original, uncommented and improved)
// A series is due if `next_checked_at` is past or NULL, and status is not 'paused' or 'completed'.

/*pub fn get_series_to_check(conn: &Connection, limit: Option<u32>) -> Result<Vec<ManhwaSeries>> {
    let now = current_timestamp();
    // Define statuses to ignore directly in the query for simplicity with params.
    // If statuses were dynamic, building the query string would be more complex.
    let ignore_status = ["paused", "completed"];

    // Simpler query construction using IN operator and binding multiple values
    // This requires enabling `sqlite_parameters_in_query` feature or similar if not default.
    // For rusqlite, we can construct the `?` placeholders dynamically.
    let mut sql = String::from("SELECT id, title, current_source_url, source_website_host, last_chapter_found_in_storage, processing_status, check_interval_minutes, last_checked_at, next_checked_at, created_at, updated_at FROM manhwa_series WHERE (next_check_at <= ?1 OR next_check_at IS NULL) AND processing_status NOT IN (");
    for (i, _) in ignore_status.iter().enumerate() {
        if i > 0 {
            sql.push_str(", ");
        }
        sql.push_str(&format!("?{}", i + 2));
    }
    sql.push_str(") ORDER BY next_checked_at ASC");

    if let Some(l) = limit {
        sql.push_str(&format!(" LIMIT {}", l));
    }

    let mut stmt = conn.prepare(&sql)?;

    // Create dynamic params for the status values
    let mut query_param: Vec<&dyn rusqlite::ToSql> = Vec::new();
    query_param.push(&now);
    for status in &ignore_status {
        query_param.push(status);
    }

    let series_iter = stmt.query_map(query_param.as_slice(), row_manhwa_series)?;

    let mut series_list = Vec::new();
    for series_result in series_iter {
        match series_result {
            Ok(series) => series_list.push(series),
            Err(e) => {
            // Log error and skip this series, or propagate. For now, log and skip.
            eprintln!("[DB] Error mapping series to check: {}", e)
            }
        }
    }
    Ok(series_list)
}*/
