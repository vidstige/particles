#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rgba8 {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
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

    pub fn to_rgba8(self, alpha: f32) -> Rgba8 {
        let alpha = alpha.clamp(0.0, 1.0);
        Rgba8::new(
            (self.red.clamp(0.0, 1.0) * alpha * 255.0).round() as u8,
            (self.green.clamp(0.0, 1.0) * alpha * 255.0).round() as u8,
            (self.blue.clamp(0.0, 1.0) * alpha * 255.0).round() as u8,
            (alpha * 255.0).round() as u8,
        )
    }
}

impl Rgba8 {
    pub const TRANSPARENT: Self = Self::new(0, 0, 0, 0);

    pub const fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    pub const fn from_rgb(red: u8, green: u8, blue: u8) -> Self {
        Self::new(red, green, blue, 255)
    }

    pub fn scale(self, scale: f32) -> Self {
        Self::new(
            (self.red as f32 * scale).round().clamp(0.0, 255.0) as u8,
            (self.green as f32 * scale).round().clamp(0.0, 255.0) as u8,
            (self.blue as f32 * scale).round().clamp(0.0, 255.0) as u8,
            (self.alpha as f32 * scale).round().clamp(0.0, 255.0) as u8,
        )
    }
}
