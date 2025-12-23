use axum::{http::StatusCode};
use image::RgbaImage;
use std::{
    collections::VecDeque, sync::{Arc, Mutex, OnceLock}, thread
};
use xcap::{Monitor};

use crate::handlers::{gifs, processor::ImageProcessor};

static FRAME_BUFFER: OnceLock<Arc<Mutex<VecDeque<Vec<u8>>>>> = OnceLock::new();
const MAX_FRAMES: usize = 24;
const FRAME_BATCH_SIZE: usize = 24;
pub fn init_frame_buffer() -> Arc<Mutex<VecDeque<Vec<u8>>>> {
    FRAME_BUFFER
        .get_or_init(|| Arc::new(Mutex::new(VecDeque::with_capacity(MAX_FRAMES))))
        .clone()
}


pub async fn start_screenshare() -> StatusCode {

    let buffer = init_frame_buffer();
    let monitor = Monitor::from_point(100, 100).unwrap();
    let (video_recorder, sx) = monitor.video_recorder().unwrap();

    thread::spawn( move || {
        loop {

            match sx.recv() {
                Ok(frame) => {

                    let raw = frame.raw;
                    let mut buf = buffer.lock().unwrap();

                    if buf.len() >= MAX_FRAMES {
                        buf.pop_front();
                    }

                    buf.push_back(raw);

                    // println!(
                    //     "🖼️ Frame salvo. Buffer contém {} frames.",
                    //     buf.len()
                    // );

                }
                Err(_) =>  { continue; }

            }

            std::thread::sleep(std::time::Duration::from_millis(32));
            
        }   


    });

    video_recorder.start().unwrap();

    StatusCode::OK

}

pub async fn get_frames() -> Result<Vec<u8>, String> {

    let buffer = FRAME_BUFFER.get();

    let buffer = buffer.unwrap();
    let mut buffer = buffer.lock().unwrap();

    let mut frames_raw: Vec<Vec<u8>> = Vec::new();
    for _ in 0..FRAME_BATCH_SIZE.min(buffer.len()) {
        if let Some(frame) = buffer.pop_front() {
            frames_raw.push(frame);
        }
    }

    let width = 1920;
    let height = 1080;

    let target_width = 640;
    let target_height = 360;

    let frames: Vec<RgbaImage> = frames_raw
        .into_iter()
        .filter_map(|raw| {
            RgbaImage::from_raw(width, height, raw).map(|img| {
                image::imageops::resize(
                    &img,
                    target_width,
                    target_height,
                    image::imageops::FilterType::Triangle
                )
            })
        })
        .collect();

    let processor = ImageProcessor::new(256);
    let tiles = processor.make_tiles_from_frames(frames);

    let payload = gifs::get_gif_payload(&tiles);
    
    Ok(payload)

}
