use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use axum_extra::extract::Multipart;
use serde::Deserialize;
use serde_json::json;

use crate::{
    Store,
    handlers::{gifs, images}, models::file::{self },
};

#[derive(Deserialize)]
pub struct LatestQuery {
    field: Option<String>, // ex: "id", "name", "width", etc.
}

pub async fn get_latest(
    State(store): State<Store>,
    Query(params): Query<LatestQuery>,
) -> impl IntoResponse {
    let map = store.lock().unwrap();

    if let Some(latest) = map.values().max_by_key(|f| f.uploaded_at) {
        match params.field.as_deref() {
            Some("id") => (StatusCode::OK, latest.id.clone()),
            Some("name") => (StatusCode::OK, latest.name.clone()),
            Some("width") => (StatusCode::OK, latest.width.to_string()),
            Some("height") => (StatusCode::OK, latest.height.to_string()),
            _ => (
                StatusCode::OK,
                format!("Último arquivo: {} ({})", latest.name, latest.id),
            ),
        }
    } else {
        (StatusCode::NOT_FOUND, "Nenhum arquivo encontrado".into())
    }
}

pub async fn get_latest_payload(State(store): State<Store>) -> Result<Vec<u8>, StatusCode> {
    let map = store.lock().unwrap();

    if let Some(file) = map.values().max_by_key(|f| f.uploaded_at) {
        match &file.content {
            file::FileContent::Static { tiles } => Ok(images::get_image_payload(tiles)),
            file::FileContent::Animated { frames } => Ok(gifs::get_gif_payload(frames))
        } 
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub async fn get_latest_meta(State(store): State<Store>) -> impl IntoResponse {
    let map = store.lock().unwrap();

    if let Some(file) = map.values().max_by_key(|f| f.uploaded_at) {
        let response = json!({
            "id": file.id,
            "width": file.width,
            "height": file.height,
            "extension": file.format
        });

        (StatusCode::OK, Json(response))
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "No files found"})),
        )
        
    }
}

pub async fn upload(
    State(store): State<Store>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let Some(field) = multipart.next_field().await.map_err(internal_error)? else {
        return Err((StatusCode::BAD_REQUEST, "Nenhum arquivo enviado".into()));
    };

    let file_name = field.file_name().unwrap_or("unnamed.bin").to_string();
    let content_type = field
        .content_type()
        .map(|v| v.to_string())
        .unwrap_or_default();

    let data = field.bytes().await.map_err(internal_error)?;

    if content_type.starts_with("image/gif") {
        gifs::upload_gif(store.clone(), &file_name, &content_type, data)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Erro while uploading GIF: {}", e),
                )
            })?;
    } else if content_type.starts_with("image/") {
        images::upload_image(store.clone(), &file_name, &content_type, data)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Error while uploading image: {}", e),
                )
            })?;
    }

    Ok(Json(json!({
        "success": true,
    })))
    
}

pub async fn get_file_by_id(Path(file_id): Path<String>, store: State<Store>) -> impl IntoResponse {

    let map = store.lock().unwrap();

    if let Some(file) = map.get(&file_id) {
        // Determina número de tiles/frames e dimensões com base no conteúdo
        let (num_items, dimensions) = 
        
        match &file.content {
            file::FileContent::Static{ tiles } => (tiles.len(), format!("{}x{}", file.width, file.height)),
            file::FileContent::Animated { frames} => {
                let num_tiles: usize = frames.iter().map(|f| f.len()).sum();
                (num_tiles, format!("{}x{}", file.width, file.height))
            }
        };

        let info = format!(
            "Arquivo: {}\nID: {}\nFormato: {}\nMIME: {}\nDimensões: {}\nNúmero de tiles/frames: {}",
            file.name,
            file.id,
            file.format,
            file.mime_type,
            dimensions,
            num_items
        );

        (StatusCode::OK, info)
    } else {
        (StatusCode::NOT_FOUND, "Arquivo não encontrado".into())
    }
}

fn internal_error<E: std::fmt::Display>(err: E) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Erro interno: {err}"),
    )
}
