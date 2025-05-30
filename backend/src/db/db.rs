use rusqlite::{Connection, Error as RusqliteError, OptionalExtension, Result, Row, params};
//use std::fmt::format;
use std::fs;
//use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
//use image::error::UnsupportedErrorKind::Format;

//#[derive(Debug)]
pub struct ManhwaSeries {
    pub id: i32,
    pub title: String,
    pub current_source_url: Option<String>,
    pub source_website_host: Option<String>,
    pub last_chapter_found_locally: Option<f32>,
    pub processing_status: String,
    pub check_interval_minutes: i32,
    pub last_checked_at: Option<i64>,
    pub next_checked_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

pub fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as i64
}

pub fn connect_db(db_path: &str) -> Result<Connection> {
    Connection::open(db_path)
}

pub fn initialize_schema(conn: &Connection, db_sql_file_path: &str) -> Result<()> {
    let schema_sql = fs::read_to_string(db_sql_file_path).map_err(|e| {
        RusqliteError::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_IOERR_READ),
            Some(format!("Failed to read schema {}: {}", db_sql_file_path, e)),
        )
    })?;
    conn.execute_batch(&schema_sql)?;
    println!(
        "[DB] Schema database from {} initialized successfully",
        db_sql_file_path
    );
    Ok(())
}

/// Add new manhwa series to the database
pub fn add_manhwa_series(
    conn: &Connection,
    title: &str,
    source_url: Option<&str>,
    interval_minutes: i32,
) -> Result<i64> {
    let now = current_timestamp();
    let host = source_url.and_then(|url_str| {
        url::Url::parse(url_str)
            .ok()
            .and_then(|url| url.host_str().map(String::from))
    });

    let initial_next_checked = now;

    conn.execute(
        "INSERT INTO manhwa_series (title, current_source_url, source_website_host, processing_status, check_interval_minutes, next_checked_at, created_at, updated_at)
         VALUES (?1, ?2, ?3, 'pending', ?4, ?5, ?6, ?7)",
        params![
            title,
            source_url,
            host,
            interval_minutes,
            initial_next_checked,
            now,
            now
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

/// Helper function to map a database row into a ManhwaSeries struct
fn row_manhwa_series(row: &Row) -> Result<ManhwaSeries> {
    Ok(ManhwaSeries {
        id: row.get(0)?,
        title: row.get(1)?,
        current_source_url: row.get(2)?,
        source_website_host: row.get(3)?,
        last_chapter_found_locally: row.get(4)?,
        processing_status: row.get(5)?,
        check_interval_minutes: row.get(6)?,
        last_checked_at: row.get(7)?,
        next_checked_at: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

/// Get single manhwa series by ID
pub fn get_manhwa_series_by_id(conn: &Connection, id: i32) -> Result<Option<ManhwaSeries>> {
    conn.query_row(
        "SELECT id, title, current_source_url, source_website_host, last_chapter_found_locally, processing_status, check_interval_minutes, last_checked_at, next_checked_at, created_at, updated_at FROM manhwa_series WHERE id = ?1",
        params![id],
        row_manhwa_series
    ).optional()
}

/// Get single manhwa series by title (UNIQUE)
pub fn get_manhwa_series_by_title(conn: &Connection, title: &str) -> Result<Option<ManhwaSeries>> {
    conn.query_row(
        "SELECT id, title, current_source_url, source_website_host, last_chapter_found_locally, processing_status, check_interval_minutes, last_checked_at, next_checked_at, created_at, updated_at FROM manhwa_series WHERE title = ?1",
        params![title],
        row_manhwa_series
    ).optional()
}

// Get all manhwa series from database
/*pub fn get_all_manhwa_series(conn: &Connection) -> Result<Vec<ManhwaSeries>> {
    let mut stmt = conn.prepare("SELECT id, title, current_source_url, source_website_host, last_chapter_found_locally, processing_status, check_interval_minutes, last_checked_at, next_checked_at, created_at, updated_at FROM manhwa_series ORDER BY title ASC")?;
    let series_iter = stmt.query_map([], row_manhwa_series)?;

    let mut series_list = Vec::new();
    for series in series_iter {
        series_list.push(series?);
    }
    Ok(series_list)
}*/

// Get manhwa series that are due for checking (next_checked_at <= now and status not paused/completed)
// Sorted by next_checked_at so the longest waiting series comes first
/*pub fn get_series_to_check(conn: &Connection, limit: Option<u32>) -> Result<Vec<ManhwaSeries>> {
    let now = current_timestamp();
    let ignore_status = ["paused", "completed"];

    let mut sql = String::from("SELECT id, title, current_source_url, source_website_host, last_chapter_found_locally, processing_status, check_interval_minutes, last_checked_at, next_checked_at, created_at, updated_at FROM manhwa_series WHERE (next_check_at <= ?1 OR next_check_at IS NULL) AND processing_status NOT IN (");
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
            Err(e) => eprintln!("[DB] Error mapping series to check: {}", e)
        }
    }
    Ok(series_list)
}*/

/// Update source URLs series
pub fn update_series_source_urls(
    conn: &Connection,
    series_id: i32,
    new_source_url: &str,
) -> Result<usize> {
    let now = current_timestamp();
    let new_host = url::Url::parse(new_source_url)
        .ok()
        .and_then(|url| url.host_str().map(String::from));
    conn.execute(
        "UPDATE manhwa_series SET current_source_url = ?1, source_website_host = ?2, updated_at = ?3 WHERE id = ?4",
        params![new_source_url, new_host, now, series_id],
    )
}

/// Update last chapter found locally
pub fn update_series_last_local_chapter(
    conn: &Connection,
    series_id: i32,
    chapter_number: Option<f32>,
) -> Result<usize> {
    let now = current_timestamp();
    conn.execute(
        "UPDATE manhwa_series SET last_chapter_found_locally = ?1, updated_at = ?2 WHERE id = ?3",
        params![chapter_number, now, series_id],
    )
}

/// Update processing status and schedule check for series
/// Usualy called after series are downloaded and parsed
pub fn update_series_check_schedule(
    conn: &Connection,
    series_id: i32,
    new_status: Option<&str>,
    new_last_checked_at: Option<i64>,
    new_next_checked_at: Option<i64>,
) -> Result<usize> {
    let now = current_timestamp();
    let series = get_manhwa_series_by_id(conn, series_id)?
        .ok_or_else(|| RusqliteError::QueryReturnedNoRows)?;

    let final_status = new_status.unwrap_or(&series.processing_status);
    let final_last_checked_at = new_last_checked_at.unwrap_or(now);

    let final_next_checked_at = match new_next_checked_at {
        Some(ts) => ts,
        None => final_last_checked_at + (series.check_interval_minutes as i64 * 60),
    };

    conn.execute(
        "UPDATE manhwa_series SET processing_status = ?1, last_checked_at = ?2, next_checked_at = ?3, updated_at = ?4 WHERE id = ?5",
        params![final_status, final_last_checked_at, final_next_checked_at, now, series_id],
    )
}

// Update series processing status
/*pub fn update_series_processing_status(conn: &Connection, series_id: i32, new_status: &str) -> Result<usize> {
    let now = current_timestamp();
    conn.execute(
        "UPDATE manhwa_series SET processing_status = ?1, updated_at = ?2 WHERE id = ?3",
        params![new_status, now, series_id],
    )
}*/

// Delete series ID from database
/*pub fn delete_series(conn: &Connection, id:i32) -> Result<usize> {
    conn.execute("DELETE FROM manhwa_series WHERE id = ?1", params![id])
}*/

// Anda mungkin ingin menambahkan fungsi lain sesuai kebutuhan, misalnya:
// - update_check_interval_minutes
// - find_series_by_source_host
// Contoh penggunaan (biasanya akan ada di main.rs atau modul logika scraper):
/*
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = "manhwa_scraper.db";
    let db_sql_file = "src/db/db.sql"; // Path ke file DDL Anda
    let conn = connect_db(db_path)?;

    initialize_schema(&conn, db_sql_file)?; // Memanggil dengan path file

    // Tambah seri baru
    let series_id = add_manhwa_series(&conn, "Solo Leveling Example", Some("https://example.com/solo-leveling"), 60)?;
    println!("Seri baru ditambahkan dengan ID: {}", series_id);

    let series_id_2 = add_manhwa_series(&conn, "Another Manhwa", None, 120)?;
     println!("Seri baru ditambahkan dengan ID: {}", series_id_2);


    // Dapatkan seri berdasarkan ID
    if let Some(series) = get_manhwa_series_by_id(&conn, series_id)? {
        println!("Seri ditemukan: {:?}", series);
        update_series_last_local_chapter(&conn, series.id, Some(10.5))?;
        println!("Updated last local chapter for {}", series.title);
    }

    // Dapatkan semua seri
    let all_series = get_all_manhwa_series(&conn)?;
    println!("\nSemua seri:");
    for s in all_series {
        println!("- Title: {}, Status: {}, Next Check: {:?}", s.title, s.processing_status, s.next_checked_at.map(|ts| SystemTime::UNIX_EPOCH + Duration::from_secs(ts as u64)));
    }

    // Dapatkan seri yang perlu dicek
    let to_check = get_series_to_check(&conn, Some(5))?; // Ambil maksimal 5
    println!("\nSeri yang perlu dicek:");
    for s in to_check {
        println!("- ID: {}, Title: {}, Next Check At: {:?}", s.id, s.title, s.next_checked_at.map(|ts| SystemTime::UNIX_EPOCH + Duration::from_secs(ts as u64)));
        // Misalkan kita selesai memproses seri ini
        update_series_check_schedule(&conn, s.id, Some("monitoring"), Some(current_unix_timestamp()), None)?; // next_checked_at akan dihitung otomatis
        println!("  Updated schedule for {}", s.title);
    }
    Ok(())
}
*/
