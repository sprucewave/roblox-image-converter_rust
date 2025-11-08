use crate::models::tile::Tile;
use chrono::{DateTime, Utc};

#[derive(Clone)]
pub enum FileContent {
    Static { tiles: Vec<Tile> },
    Animated { frames: Vec<Vec<Tile>>}
}

#[derive(Clone)]
pub struct StoredFile {
    pub id: String,
    pub name: String,
    pub mime_type: String,
    pub format: String,
    pub width: u32,
    pub height: u32,
    pub content: FileContent,
    pub uploaded_at: DateTime<Utc>,
}
