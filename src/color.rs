use tiny_skia::Color as TinyColor;

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

    pub fn overflow_scale(self) -> f32 {
        self.red.max(self.green).max(self.blue).max(1.0)
    }

    pub fn to_tiny_color(self) -> TinyColor {
        let scale = self.overflow_scale();
        TinyColor::from_rgba(
            (self.red / scale).clamp(0.0, 1.0),
            (self.green / scale).clamp(0.0, 1.0),
            (self.blue / scale).clamp(0.0, 1.0),
            1.0,
        )
        .unwrap()
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

    #[test]
    fn to_tiny_color_scales_overflow_channels() {
        let color = Color::new(2.0, 1.0, 0.5).to_tiny_color();

        assert_eq!(color.red(), 1.0);
        assert_eq!(color.green(), 0.5);
        assert_eq!(color.blue(), 0.25);
        assert_eq!(color.alpha(), 1.0);
    }

    #[test]
    fn overflow_scale_matches_brightest_channel() {
        assert_eq!(Color::new(2.0, 1.0, 0.5).overflow_scale(), 2.0);
        assert_eq!(Color::new(0.5, 0.25, 0.125).overflow_scale(), 1.0);
    }
}
