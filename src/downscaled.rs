use glam::Vec3;

use crate::{bitmap::Bitmap, color::Color, render::Render, resolution::Resolution};

pub struct Downscaled<R> {
    pub inner: R,
    pub scale: u32,
}

impl<R: Render> Render for Downscaled<R> {
    fn render(&self, target: &mut Bitmap, positions: &[Option<Vec3>], colors: &[Color]) {
        let scale = self.scale as f32;
        let small_res = Resolution::new(
            target.width() / self.scale,
            target.height() / self.scale,
        );
        let mut small = Bitmap::new(small_res);
        let scaled_positions: Vec<Option<Vec3>> = positions
            .iter()
            .map(|p| p.map(|v| Vec3::new(v.x / scale, v.y / scale, v.z)))
            .collect();
        self.inner.render(&mut small, &scaled_positions, colors);
        blit_upscaled_additive(&small, target);
    }
}

fn blit_upscaled_additive(small: &Bitmap, dest: &mut Bitmap) {
    let sw = small.width() as usize;
    let sh = small.height() as usize;
    let dw = dest.width() as usize;
    let dh = dest.height() as usize;
    let scale_x = sw as f32 / dw as f32;
    let scale_y = sh as f32 / dh as f32;

    let sdata = small.data();
    let ddata = dest.data_mut();

    for y in 0..dh {
        let sy = (y as f32 + 0.5) * scale_y - 0.5;
        let iy = sy.floor() as i32;
        let fy = sy - iy as f32;
        let iy0 = iy.clamp(0, sh as i32 - 1) as usize;
        let iy1 = (iy + 1).clamp(0, sh as i32 - 1) as usize;

        for x in 0..dw {
            let sx = (x as f32 + 0.5) * scale_x - 0.5;
            let ix = sx.floor() as i32;
            let fx = sx - ix as f32;
            let ix0 = ix.clamp(0, sw as i32 - 1) as usize;
            let ix1 = (ix + 1).clamp(0, sw as i32 - 1) as usize;

            let s00 = (iy0 * sw + ix0) * 4;
            let s10 = (iy0 * sw + ix1) * 4;
            let s01 = (iy1 * sw + ix0) * 4;
            let s11 = (iy1 * sw + ix1) * 4;
            let d = (y * dw + x) * 4;

            for c in 0..4 {
                let p00 = sdata[s00 + c] as f32;
                let p10 = sdata[s10 + c] as f32;
                let p01 = sdata[s01 + c] as f32;
                let p11 = sdata[s11 + c] as f32;
                let top = p00 * (1.0 - fx) + p10 * fx;
                let bot = p01 * (1.0 - fx) + p11 * fx;
                let val = top * (1.0 - fy) + bot * fy;
                ddata[d + c] = ddata[d + c].saturating_add(val as u8);
            }
        }
    }
}
