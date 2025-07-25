// Retrieves all manhwa series from the database, ordered by title.
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
