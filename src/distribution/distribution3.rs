use glam::Vec3;

use crate::rng::Rng;

pub trait Distribution3 {
    fn sample(&mut self, rng: &mut Rng) -> Vec3;
}

impl<D: Distribution3 + ?Sized> Distribution3 for Box<D> {
    fn sample(&mut self, rng: &mut Rng) -> Vec3 {
        self.as_mut().sample(rng)
    }
}

pub fn collect<D: Distribution3>(distribution: &mut D, count: usize, rng: &mut Rng) -> Vec<Vec3> {
    (0..count).map(|_| distribution.sample(rng)).collect()
}
