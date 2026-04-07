use crate::{color::Rgba8, resolution::Resolution};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Bitmap {
    resolution: Resolution,
    data: Vec<u8>,
}

impl Bitmap {
    pub fn new(resolution: Resolution) -> Self {
        let len = resolution.area() * 4;
        Self {
            resolution,
            data: vec![0; len],
        }
    }

    pub fn width(&self) -> u32 {
        self.resolution.width
    }

    pub fn height(&self) -> u32 {
        self.resolution.height
    }

    pub fn resolution(&self) -> &Resolution {
        &self.resolution
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn fill(&mut self, color: Rgba8) {
        for pixel in self.data.chunks_exact_mut(4) {
            pixel.copy_from_slice(&[color.red, color.green, color.blue, color.alpha]);
        }
    }

    pub fn pixel(&self, x: u32, y: u32) -> Option<Rgba8> {
        let index = self.pixel_index(x, y)?;
        Some(Rgba8::new(
            self.data[index],
            self.data[index + 1],
            self.data[index + 2],
            self.data[index + 3],
        ))
    }

    pub(crate) fn pixel_index(&self, x: u32, y: u32) -> Option<usize> {
        if x >= self.width() || y >= self.height() {
            return None;
        }
        Some((y as usize * self.width() as usize + x as usize) * 4)
    }

    pub(crate) fn data_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }
}

#[cfg(test)]
mod tests {
    use super::Bitmap;
    use crate::{color::Rgba8, resolution::Resolution};

    #[test]
    fn fill_sets_every_pixel() {
        let mut bitmap = Bitmap::new(Resolution::new(2, 2));
        let color = Rgba8::new(1, 2, 3, 4);

        bitmap.fill(color);

        assert_eq!(bitmap.pixel(0, 0), Some(color));
        assert_eq!(bitmap.pixel(1, 0), Some(color));
        assert_eq!(bitmap.pixel(0, 1), Some(color));
        assert_eq!(bitmap.pixel(1, 1), Some(color));
    }

    #[test]
    fn pixel_rejects_out_of_bounds_positions() {
        let bitmap = Bitmap::new(Resolution::new(2, 2));

        assert_eq!(bitmap.pixel(2, 0), None);
        assert_eq!(bitmap.pixel(0, 2), None);
    }

}
