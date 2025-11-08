use std::io::Cursor;

use crate::{
    Store,
    handlers::processor,
    models::{file::{FileContent, StoredFile}, tile::Tile},
};
use axum::http::StatusCode;
use byteorder::{BigEndian, WriteBytesExt};
use bytes::Bytes;
use image::{
    AnimationDecoder, ImageDecoder, ImageFormat, ImageReader, RgbaImage, codecs::gif::GifDecoder,
};

pub fn fast_dimensions(data: &[u8]) -> Result<(u32, u32, String), String> {
    let reader = ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .map_err(|e| e.to_string())?;

    let format = reader
        .format()
        .ok_or("Formato da imagem não pôde ser identificado")?;

    let cursor = Cursor::new(data);
    let decoder = GifDecoder::new(cursor).map_err(|e| e.to_string())?;
    let dimensions = decoder.dimensions(); // retorna (u32, u32)
    Ok((dimensions.0, dimensions.1, format_to_string(format)))
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

pub fn gif_to_rgba_frames(data: &[u8]) -> Result<Vec<RgbaImage>, String> {
    let cursor = Cursor::new(data);

    let decoder = image::codecs::gif::GifDecoder::new(cursor).map_err(|e| e.to_string())?;

    let frames = decoder.into_frames();
    let frames = frames.collect_frames().map_err(|e| e.to_string())?;

    let rgba_frames: Vec<RgbaImage> = frames
        .into_iter()
        .map(|f| f.into_buffer()) // frame.into_buffer() retorna RgbaImage
        .collect();

    Ok(rgba_frames)
}

pub async fn upload_gif(
    store: Store,
    file_name: &str,
    mime_type: &str,
    data: Bytes,
) -> Result<StatusCode, String> {
    let id = generate_id_from_bytes(&data);
    println!("{}", id);
    let (width, height, format) = fast_dimensions(&data)?;

    let frames = tokio::task::spawn_blocking(move || process_gif_sync(&data))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())?;

    let file = StoredFile {
        id,
        name: file_name.into(),
        format,
        width: width,
        height: height,
        content: FileContent::Animated { frames },
        mime_type: mime_type.into(),
        uploaded_at: chrono::Utc::now(),
    };

    store.lock().unwrap().insert(file.id.clone(), file);

    Ok(StatusCode::OK)
}

fn process_gif_sync(data: &[u8]) -> Result<Vec<Vec<Tile>>, String> {
    let frames = gif_to_rgba_frames(data)?;

    let processor = processor::ImageProcessor::new(256);
    let tiles = processor.make_tiles_from_frames(frames);

    Ok(tiles)
}

pub fn get_gif_payload(frames: &[Vec<Tile>]) -> Vec<u8> {
    let mut payload = Vec::new();

    payload
        .write_u32::<BigEndian>(frames.len() as u32)
        .unwrap();

    for frame in frames {
        payload.write_u32::<BigEndian>(frame.len() as u32).unwrap();

        for tile in frame {
            payload.write_u32::<BigEndian>(tile.x).unwrap();
            payload.write_u32::<BigEndian>(tile.y).unwrap();
            payload.write_u32::<BigEndian>(tile.width).unwrap();
            payload.write_u32::<BigEndian>(tile.height).unwrap();
            payload
                .write_u32::<BigEndian>(tile.data.len() as u32)
                .unwrap();
            payload.extend_from_slice(&tile.data);
        }
    }

    payload
}
