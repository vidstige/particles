#[derive(Clone)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl Resolution {
    pub fn new(width: u32, height: u32) -> Resolution {
        Resolution { width, height }
    }
    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }
    pub fn area(&self) -> usize {
        (self.width * self.height) as usize
    }
}
