use glam::Vec3;

use super::Distribution3;
use crate::rng::Rng;

#[derive(Debug)]
pub struct Constant {
    value: Vec3,
}

impl Constant {
    pub fn new(value: Vec3) -> Self {
        Self { value }
    }
}

impl Distribution3 for Constant {
    fn sample(&mut self, _rng: &mut Rng) -> Vec3 {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec3;

    use super::Constant;
    use crate::{distribution::collect, rng::Rng};

    #[test]
    fn constant_returns_the_same_value_for_every_sample() {
        let mut rng = Rng::new(0x1234_5678);
        let value = Vec3::new(1.0, 2.0, 3.0);

        let points = collect(&mut Constant::new(value), 4, &mut rng);

        assert_eq!(points, vec![value; 4]);
    }
}
