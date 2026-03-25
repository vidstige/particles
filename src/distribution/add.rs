use glam::Vec3;

use super::Distribution3;
use crate::rng::Rng;

#[derive(Debug)]
pub struct Add<L, R> {
    left: L,
    right: R,
}

impl<L, R> Add<L, R> {
    pub fn new(left: L, right: R) -> Self {
        Self { left, right }
    }
}

impl<L: Distribution3, R: Distribution3> Distribution3 for Add<L, R> {
    fn sample(&mut self, rng: &mut Rng) -> Vec3 {
        self.left.sample(rng) + self.right.sample(rng)
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec3;

    use super::Add;
    use crate::{
        distribution::{collect, Constant},
        rng::Rng,
    };

    #[test]
    fn add_combines_samples_from_both_distributions() {
        let mut rng = Rng::new(0x1234_5678);

        let points = collect(
            &mut Add::new(
                Constant::new(Vec3::new(1.0, 2.0, 3.0)),
                Constant::new(Vec3::new(0.5, 0.25, -1.0)),
            ),
            4,
            &mut rng,
        );

        assert_eq!(points, vec![Vec3::new(1.5, 2.25, 2.0); 4]);
    }
}
