#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
}

impl Color {
    pub const BLACK: Self = Self::new(0.0, 0.0, 0.0);

    pub const fn new(red: f32, green: f32, blue: f32) -> Self {
        Self { red, green, blue }
    }

    pub fn from_rgb8(red: u8, green: u8, blue: u8) -> Self {
        Self::new(
            red as f32 / 255.0,
            green as f32 / 255.0,
            blue as f32 / 255.0,
        )
    }

    pub fn overflow_scale(self) -> f32 {
        self.red.max(self.green).max(self.blue).max(1.0)
    }
}
