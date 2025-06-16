pub async fn manga_updates_handler() -> &'static str {
    "Manga updates handler"
}

pub async fn get_manga_updates() -> axum::Json<Vec<&'static str>> {
    axum::Json(vec!["Manga update 1", "Manga update 2", "Manga update 3"])
}
