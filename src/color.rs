use tiny_skia::Color as TinyColor;

/// RGB accumulation color for the renderer.
///
/// We don't use `tiny_skia::Color` here because it stores normalized `0..=1`
/// channels and clamps through its safe constructors and setters, while the
/// glow pass needs an unbounded intermediate RGB value before quantizing back
/// into the pixmap.
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

    pub fn from_tiny_color(color: TinyColor) -> Self {
        let alpha = color.alpha() * 255.0;
        Self::new(color.red(), color.green(), color.blue()) * alpha
    }

    pub fn from_rgb8(red: u8, green: u8, blue: u8) -> Self {
        Self::new(red as f32, green as f32, blue as f32)
    }

    pub fn is_black(self) -> bool {
        self == Self::BLACK
    }

    pub fn lerp(self, other: Self, t: f32) -> Self {
        self * (1.0 - t) + other * t
    }
}

impl std::ops::Add for Color {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(
            self.red + rhs.red,
            self.green + rhs.green,
            self.blue + rhs.blue,
        )
    }
}

impl std::ops::AddAssign for Color {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl std::ops::Mul<f32> for Color {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.red * rhs, self.green * rhs, self.blue * rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::Color;

    #[test]
    fn lerp_interpolates_channels() {
        let left = Color::new(10.0, 20.0, 30.0);
        let right = Color::new(30.0, 10.0, 50.0);

        assert_eq!(left.lerp(right, 0.25), Color::new(15.0, 17.5, 35.0));
    }
}
