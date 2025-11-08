use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{handlers::files, models::file::StoredFile};
use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};

pub type Store = Arc<Mutex<HashMap<String, StoredFile>>>;

pub fn file_routes() -> Router<Store> {
    Router::new()
        .route("/latest", get(files::get_latest))
        .route("/latest/payload", get(files::get_latest_payload))
        .route("/latest/meta", get(files::get_latest_meta))
        .route("/upload", post(files::upload))
        .layer(DefaultBodyLimit::disable())
        .route("/{id}", get(files::get_file_by_id))
}
