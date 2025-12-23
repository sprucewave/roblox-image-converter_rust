use crate::{screenshare::screenshare_main::{self}};
use axum::{
    Router,
    routing::{get},
};


pub fn routes() -> Router {
    Router::new()
        .route("/start", get(screenshare_main::start_screenshare))
        .route("/get_frames", get(screenshare_main::get_frames))
}