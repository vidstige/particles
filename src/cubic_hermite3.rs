use glam::Vec3;

#[derive(Clone, Copy, Debug)]
pub struct CubicHermite3 {
    start: Vec3,
    start_tangent: Vec3,
    end: Vec3,
    end_tangent: Vec3,
}

impl CubicHermite3 {
    pub fn new(start: Vec3, start_tangent: Vec3, end: Vec3, end_tangent: Vec3) -> Self {
        Self {
            start,
            start_tangent,
            end,
            end_tangent,
        }
    }

    pub fn sample(&self, t: f32) -> Vec3 {
        let t2 = t * t;
        let t3 = t2 * t;
        let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
        let h10 = t3 - 2.0 * t2 + t;
        let h01 = -2.0 * t3 + 3.0 * t2;
        let h11 = t3 - t2;

        self.start * h00 + self.start_tangent * h10 + self.end * h01 + self.end_tangent * h11
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec3;

    use super::CubicHermite3;

    #[test]
    fn cubic_hermite3_hits_endpoints() {
        let curve = CubicHermite3::new(
            Vec3::new(-1.0, 2.0, 0.5),
            Vec3::new(0.5, -0.25, 1.0),
            Vec3::new(3.0, -4.0, 2.0),
            Vec3::new(-1.0, 0.75, 0.0),
        );

        assert_eq!(curve.sample(0.0), Vec3::new(-1.0, 2.0, 0.5));
        assert_eq!(curve.sample(1.0), Vec3::new(3.0, -4.0, 2.0));
    }

    #[test]
    fn cubic_hermite3_stays_linear_for_matching_tangents() {
        let curve = CubicHermite3::new(Vec3::ZERO, Vec3::X, Vec3::X, Vec3::X);

        assert_eq!(curve.sample(0.25), Vec3::new(0.25, 0.0, 0.0));
        assert_eq!(curve.sample(0.5), Vec3::new(0.5, 0.0, 0.0));
        assert_eq!(curve.sample(0.75), Vec3::new(0.75, 0.0, 0.0));
    }
}
