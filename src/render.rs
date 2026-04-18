use glam::Vec3;

use crate::{bitmap::Bitmap, color::Color};

pub trait Render {
    fn render(&self, target: &mut Bitmap, positions: &[Option<Vec3>], colors: &[Color]);
}
