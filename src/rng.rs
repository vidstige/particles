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
    fn produces_deterministic_sequence() {
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
}
