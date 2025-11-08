mod handlers;
mod models;
mod routes;

use crate::models::file::StoredFile;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use axum::{Router, routing::get};

pub type Store = Arc<Mutex<HashMap<String, StoredFile>>>;

#[tokio::main]

async fn main() {
    let store = Arc::new(Mutex::new(HashMap::<String, StoredFile>::new()));

    let app = Router::new()
        .nest("/files", routes::file_routes())
        .with_state(store.clone())
        .route("/", get(root));

    let address = "0.0.0.0:5000";
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Rust server up!"
}
