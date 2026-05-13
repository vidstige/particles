use std::ops::{Add, Mul, Sub};
use std::str::FromStr;

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

const fn hex_digit(c: u8) -> u8 {
    match c {
        b'0'..=b'9' => c - b'0',
        b'a'..=b'f' => c - b'a' + 10,
        b'A'..=b'F' => c - b'A' + 10,
        _ => panic!("invalid hex digit"),
    }
}

const fn hex_byte(hi: u8, lo: u8) -> u8 {
    hex_digit(hi) << 4 | hex_digit(lo)
}

impl Color {
    pub const BLACK: Self = Self::new(0.0, 0.0, 0.0);

    pub const fn new(red: f32, green: f32, blue: f32) -> Self {
        Self { red, green, blue }
    }

    pub const fn from_rgb8(red: u8, green: u8, blue: u8) -> Self {
        Self::new(
            red as f32 / 255.0,
            green as f32 / 255.0,
            blue as f32 / 255.0,
        )
    }

    pub const fn from_hex(s: &str) -> Self {
        let b = s.as_bytes();
        let start = if b[0] == b'#' { 1 } else { 0 };
        if b.len() - start != 6 {
            panic!("expected 6 hex digits");
        }
        Self::from_rgb8(
            hex_byte(b[start], b[start + 1]),
            hex_byte(b[start + 2], b[start + 3]),
            hex_byte(b[start + 4], b[start + 5]),
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

impl FromStr for Color {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hex = s.strip_prefix('#').unwrap_or(s);
        if hex.len() != 6 {
            return Err(format!("expected 6 hex digits, got {:?}", s));
        }
        let r = u8::from_str_radix(&hex[0..2], 16).map_err(|e| e.to_string())?;
        let g = u8::from_str_radix(&hex[2..4], 16).map_err(|e| e.to_string())?;
        let b = u8::from_str_radix(&hex[4..6], 16).map_err(|e| e.to_string())?;
        Ok(Self::from_rgb8(r, g, b))
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

impl Default for Color {
    fn default() -> Self { Self::BLACK }
}

impl Add for Color {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.red + rhs.red, self.green + rhs.green, self.blue + rhs.blue)
    }
}

impl Sub for Color {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.red - rhs.red, self.green - rhs.green, self.blue - rhs.blue)
    }
}

impl Mul<f32> for Color {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self::new(self.red * rhs, self.green * rhs, self.blue * rhs)
    }
}
