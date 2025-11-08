use image::{RgbaImage, imageops};
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};

use crate::models::tile::Tile;

pub struct ImageProcessor {
    tile_size: u32,
}

impl ImageProcessor {
    pub fn new(tile_size: u32) -> Self {
        Self { tile_size }
    }

    pub fn make_tiles(&self, img: &RgbaImage) -> Vec<Tile> {
        let width = img.width();
        let height = img.height();

        let tiles_x = (width + self.tile_size - 1) / self.tile_size;
        let tiles_y = (height + self.tile_size - 1) / self.tile_size;
        // let total_tiles = (tiles_x * tiles_y) as usize;

        let tile_coords: Vec<(u32, u32)> = (0..tiles_y)
            .flat_map(|ty| (0..tiles_x).map(move |tx| (tx * self.tile_size, ty * self.tile_size)))
            .collect();

        tile_coords
            .par_iter()
            .map(|(x, y)| self.process_tile(img, *x, *y))
            .collect()
    }

    pub fn make_tiles_from_frames(&self, frames: Vec<RgbaImage>) -> Vec<Vec<Tile>> {
        frames
            .into_par_iter() // roda em paralelo
            .map(|frame| self.make_tiles(&frame))
            .collect()
    }

    pub fn process_tile(&self, img: &RgbaImage, x: u32, y: u32) -> Tile {
        let width = img.width();
        let height = img.height();

        let tile_w = std::cmp::min(self.tile_size, width - x);
        let tile_h = std::cmp::min(self.tile_size, height - y);

        let tile_view = imageops::crop_imm(img, x, y, tile_w, tile_h);

        let qoi_data = qoi::encode_to_vec(tile_view.to_image().as_raw(), tile_w, tile_h)
            .expect("QOI encoding failed");

        Tile {
            x,
            y,
            width: tile_w,
            height: tile_h,
            data: qoi_data,
        }
    }
}
