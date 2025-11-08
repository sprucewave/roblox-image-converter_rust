#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Tile {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

#[allow(dead_code)]
impl Tile {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            data: Vec::new(),
        }
    }
}
