#[derive(Clone, Copy, Debug, Default, PartialEq)]
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
}

#[cfg(test)]
mod tests {
    use super::Color;

    #[test]
    fn new_preserves_overflow_channels() {
        assert_eq!(Color::new(2.0, 1.5, 0.25), Color::new(2.0, 1.5, 0.25));
    }

    #[test]
    fn from_rgb8_scales_bytes_to_unit_range() {
        assert_eq!(Color::from_rgb8(255, 128, 0), Color::new(1.0, 128.0 / 255.0, 0.0));
    }
}
