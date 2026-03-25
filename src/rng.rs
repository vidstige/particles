pub struct Rng {
    state: u32,
}

impl Rng {
    pub fn new(seed: u32) -> Self {
        Self { state: seed }
    }

    pub fn next_u32(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        self.state
    }

    pub fn next_f32(&mut self) -> f32 {
        self.next_u32() as f32 / u32::MAX as f32
    }

    pub fn next_f32_in(&mut self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.next_f32()
    }

    pub fn next_index(&mut self, upper: usize) -> usize {
        assert!(upper > 0);
        ((self.next_u32() as u64 * upper as u64) >> 32) as usize
    }

    pub fn next_gaussian(&mut self) -> f32 {
        let u1 = self.next_f32().max(f32::MIN_POSITIVE);
        let u2 = self.next_f32();
        let radius = (-2.0 * u1.ln()).sqrt();
        let angle = std::f32::consts::TAU * u2;
        radius * angle.cos()
    }
}

#[cfg(test)]
mod tests {
    use super::Rng;

    #[test]
    fn next_u32_is_stable_for_fixed_seed() {
        let mut rng = Rng::new(0x1234_5678);
        let values = [
            rng.next_u32(),
            rng.next_u32(),
            rng.next_u32(),
            rng.next_u32(),
        ];
        assert_eq!(
            values,
            [1_967_335_287, 3_442_499_178, 635_173_569, 1_264_358_700]
        );
    }

    #[test]
    fn next_f32_in_and_next_index_are_stable_for_fixed_seed() {
        let mut rng = Rng::new(0x1234_5678);
        let values = [
            rng.next_f32_in(-1.0, 1.0),
            rng.next_index(6) as f32,
            rng.next_f32_in(2.0, 4.0),
            rng.next_index(20) as f32,
        ];

        assert_eq!(values, [-0.083_888_11, 4.0, 2.295_775_7, 5.0]);
    }
}
