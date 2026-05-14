use glam::Vec2;

use crate::{color::Color, field::Field, resolution::Resolution};

pub trait Sample {
    type Output;
    fn sample(&self, point: Vec2) -> Self::Output;
}

pub fn sample_at_resolution<S: Sample>(sample: S, resolution: Resolution, size: Vec2) -> Field<S::Output>
where
    S::Output: Clone + Default,
{
    let mut field = Field::new(resolution.clone(), size, S::Output::default());
    for y in 0..resolution.height as usize {
        for x in 0..resolution.width as usize {
            let pos = field.sample(x, y);
            field.set(x, y, sample.sample(pos / size));
        }
    }
    field
}

pub struct Aurora;

impl Sample for Aurora {
    type Output = Color;
    fn sample(&self, point: Vec2) -> Color {
        let t = point.y.clamp(0.0, 1.0);
        if t < 0.25 {
            Color::from_hex("#252261")
        } else if t < 0.50 {
            Color::from_hex("#6E7CA8")
        } else if t < 0.75 {
            Color::from_hex("#DD86B0")
        } else {
            Color::from_hex("#B4A5B9")
        }
    }
}
