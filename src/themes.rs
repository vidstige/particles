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

fn gradient(stops: &[(f32, Color)], t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    for i in 1..stops.len() {
        let (t0, c0) = stops[i - 1];
        let (t1, c1) = stops[i];
        if t <= t1 {
            let local_t = (t - t0) / (t1 - t0);
            return c0 * (1.0 - local_t) + c1 * local_t;
        }
    }
    stops.last().map(|(_, c)| *c).unwrap_or(Color::BLACK)
}

pub struct Cosmos;

impl Sample for Cosmos {
    type Output = Color;
    fn sample(&self, point: Vec2) -> Color {
        let t = point.y.clamp(0.0, 1.0);
        gradient(&[
            (0.00, Color::from_hex("#0D1627")), // dark bg
            (0.11, Color::from_hex("#0D1627")), //   |
            (0.18, Color::from_hex("#394E80")), //   bar: blue-purple
            (0.25, Color::from_hex("#0D1627")), //   |
            (0.40, Color::from_hex("#1F2051")), // dark blue bg
            (0.43, Color::from_hex("#1F2051")), //   |
            (0.50, Color::from_hex("#8D619E")), //   bar: purple
            (0.57, Color::from_hex("#1F2051")), //   |
            (0.71, Color::from_hex("#0D1627")), // dark bg
            (0.73, Color::from_hex("#0D1627")), //   |
            (0.80, Color::from_hex("#7B5970")), //   bar: pink-purple
            (0.87, Color::from_hex("#0D1627")), //   |
            (1.00, Color::from_hex("#0D1627")), // dark end
        ], t)
    }
}
