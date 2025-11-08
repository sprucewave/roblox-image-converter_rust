use std::{io::Cursor, time::Instant, u32};

use axum::http::StatusCode;
use byteorder::{BigEndian, WriteBytesExt};
use bytes::Bytes;
use image::{ImageFormat, ImageReader};

use crate::{
    Store,
    handlers::processor::ImageProcessor,
    models::{file::{FileContent, StoredFile}, tile::Tile},
};

pub fn fast_dimensions(data: &[u8]) -> Result<(u32, u32, String), String> {
    let reader = ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .map_err(|e| e.to_string())?;

    let format = reader
        .format()
        .ok_or("Formato da imagem não pôde ser identificado")?;
    let (width, height) = reader.into_dimensions().map_err(|e| e.to_string())?;

    Ok((width, height, format_to_string(format)))
}

fn format_to_string(format: ImageFormat) -> String {
    match format {
        ImageFormat::Png => "png",
        ImageFormat::Jpeg => "jpeg",
        ImageFormat::Gif => "gif",
        ImageFormat::Bmp => "bmp",
        ImageFormat::Ico => "ico",
        ImageFormat::Tiff => "tiff",
        ImageFormat::WebP => "webp",
        _ => "unknown",
    }
    .to_string()
}

pub fn generate_id_from_bytes(data: &[u8]) -> String {
    let hash = blake3::hash(data);
    hash.to_hex().to_string()
}


pub async fn upload_image(
    store: Store,
    file_name: &str,
    mime_type: &str,
    data: Bytes,
) -> Result<StatusCode, String> {
    let start = Instant::now();
    let id = generate_id_from_bytes(&data);
    println!("{}", id);

    let (width, height, format) = fast_dimensions(&data)?;

    let tiles = tokio::task::spawn_blocking(move || process_image_sync(&data))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())?;

    println!("Tile generation took {:?}", start.elapsed());

    let file = StoredFile {
        id,
        name: file_name.into(),
        format,
        width: width,
        height: height,
        content: FileContent::Static { tiles },
        mime_type: mime_type.into(),
        uploaded_at: chrono::Utc::now(),
    };

    store.lock().unwrap().insert(file.id.clone(), file);

    Ok(StatusCode::OK)
}

fn process_image_sync(data: &[u8]) -> Result<Vec<Tile>, String> {
    let img = image::load_from_memory(data)
        .map_err(|e| e.to_string())?
        .to_rgba8();

    let processor = ImageProcessor::new(256);
    let tiles = processor.make_tiles(&img);

    Ok(tiles)
}

pub fn get_image_payload(tiles: &[Tile]) -> Vec<u8> {
    let mut payload = Vec::new();

    payload
        .write_u32::<BigEndian>(tiles.len() as u32)
        .unwrap();

    for tile in tiles {
        payload.write_u32::<BigEndian>(tile.x).unwrap();
        payload.write_u32::<BigEndian>(tile.y).unwrap();
        payload.write_u32::<BigEndian>(tile.width).unwrap();
        payload.write_u32::<BigEndian>(tile.height).unwrap();
        payload
            .write_u32::<BigEndian>(tile.data.len() as u32)
            .unwrap();
        payload.extend_from_slice(&tile.data);
    }

    payload
}
