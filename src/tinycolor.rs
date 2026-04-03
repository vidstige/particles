use crate::color::Color;
use tiny_skia::Color as TinyColor;

pub fn to_tiny_color(color: Color) -> TinyColor {
    TinyColor::from_rgba(
        color.red.clamp(0.0, 1.0),
        color.green.clamp(0.0, 1.0),
        color.blue.clamp(0.0, 1.0),
        1.0,
    )
    .unwrap()
}

pub fn with_alpha(color: TinyColor, alpha: f32) -> TinyColor {
    let mut color = color;
    color.set_alpha(alpha);
    color
}
