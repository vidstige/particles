use glam::Vec4;

const GRADIENTS_4D: [[f32; 4]; 32] = [
    [0.0, 1.0, 1.0, 1.0],
    [0.0, 1.0, 1.0, -1.0],
    [0.0, 1.0, -1.0, 1.0],
    [0.0, 1.0, -1.0, -1.0],
    [0.0, -1.0, 1.0, 1.0],
    [0.0, -1.0, 1.0, -1.0],
    [0.0, -1.0, -1.0, 1.0],
    [0.0, -1.0, -1.0, -1.0],
    [1.0, 0.0, 1.0, 1.0],
    [1.0, 0.0, 1.0, -1.0],
    [1.0, 0.0, -1.0, 1.0],
    [1.0, 0.0, -1.0, -1.0],
    [-1.0, 0.0, 1.0, 1.0],
    [-1.0, 0.0, 1.0, -1.0],
    [-1.0, 0.0, -1.0, 1.0],
    [-1.0, 0.0, -1.0, -1.0],
    [1.0, 1.0, 0.0, 1.0],
    [1.0, 1.0, 0.0, -1.0],
    [1.0, -1.0, 0.0, 1.0],
    [1.0, -1.0, 0.0, -1.0],
    [-1.0, 1.0, 0.0, 1.0],
    [-1.0, 1.0, 0.0, -1.0],
    [-1.0, -1.0, 0.0, 1.0],
    [-1.0, -1.0, 0.0, -1.0],
    [1.0, 1.0, 1.0, 0.0],
    [1.0, 1.0, -1.0, 0.0],
    [1.0, -1.0, 1.0, 0.0],
    [1.0, -1.0, -1.0, 0.0],
    [-1.0, 1.0, 1.0, 0.0],
    [-1.0, 1.0, -1.0, 0.0],
    [-1.0, -1.0, 1.0, 0.0],
    [-1.0, -1.0, -1.0, 0.0],
];

fn hash(seed: u32, coordinates: &[i32]) -> u32 {
    let mut value = seed ^ 0x9e37_79b9;

    for coordinate in coordinates {
        value ^= *coordinate as u32;
        value = value.wrapping_mul(0x85eb_ca6b);
        value ^= value >> 13;
        value = value.wrapping_mul(0xc2b2_ae35);
        value ^= value >> 16;
    }

    value
}

fn gradient(seed: u32, cell: [i32; 4]) -> Vec4 {
    GRADIENTS_4D[hash(seed, &cell) as usize % GRADIENTS_4D.len()].into()
}

fn contribution(seed: u32, cell: [i32; 4], offset: Vec4) -> f32 {
    let t = 0.6 - offset.length_squared();
    if t <= 0.0 {
        return 0.0;
    }
    let t2 = t * t;
    t2 * t2 * gradient(seed, cell).dot(offset)
}

fn simplex4(seed: u32, point: Vec4) -> f32 {
    const F4: f32 = 0.309_016_97;
    const G4: f32 = 0.138_196_6;

    let skew = point.dot(Vec4::ONE) * F4;
    let i = (point.x + skew).floor() as i32;
    let j = (point.y + skew).floor() as i32;
    let k = (point.z + skew).floor() as i32;
    let l = (point.w + skew).floor() as i32;
    let unskew = (i + j + k + l) as f32 * G4;
    let origin = Vec4::new(
        i as f32 - unskew,
        j as f32 - unskew,
        k as f32 - unskew,
        l as f32 - unskew,
    );
    let local0 = point - origin;

    let mut rank = [0; 4];
    for a in 0..4 {
        for b in (a + 1)..4 {
            if local0[a] > local0[b] {
                rank[a] += 1;
            } else {
                rank[b] += 1;
            }
        }
    }

    let step1 = [
        (rank[0] >= 3) as i32,
        (rank[1] >= 3) as i32,
        (rank[2] >= 3) as i32,
        (rank[3] >= 3) as i32,
    ];
    let step2 = [
        (rank[0] >= 2) as i32,
        (rank[1] >= 2) as i32,
        (rank[2] >= 2) as i32,
        (rank[3] >= 2) as i32,
    ];
    let step3 = [
        (rank[0] >= 1) as i32,
        (rank[1] >= 1) as i32,
        (rank[2] >= 1) as i32,
        (rank[3] >= 1) as i32,
    ];

    let local1 = local0
        - Vec4::new(
            step1[0] as f32,
            step1[1] as f32,
            step1[2] as f32,
            step1[3] as f32,
        )
        + Vec4::splat(G4);
    let local2 = local0
        - Vec4::new(
            step2[0] as f32,
            step2[1] as f32,
            step2[2] as f32,
            step2[3] as f32,
        )
        + Vec4::splat(2.0 * G4);
    let local3 = local0
        - Vec4::new(
            step3[0] as f32,
            step3[1] as f32,
            step3[2] as f32,
            step3[3] as f32,
        )
        + Vec4::splat(3.0 * G4);
    let local4 = local0 - Vec4::ONE + Vec4::splat(4.0 * G4);

    27.0 * (contribution(seed, [i, j, k, l], local0)
        + contribution(
            seed,
            [i + step1[0], j + step1[1], k + step1[2], l + step1[3]],
            local1,
        )
        + contribution(
            seed,
            [i + step2[0], j + step2[1], k + step2[2], l + step2[3]],
            local2,
        )
        + contribution(
            seed,
            [i + step3[0], j + step3[1], k + step3[2], l + step3[3]],
            local3,
        )
        + contribution(seed, [i + 1, j + 1, k + 1, l + 1], local4))
}

fn scaled_value(value: f32, strength: f32) -> f32 {
    (value * strength).clamp(-1.0, 1.0)
}

#[derive(Clone, Copy, Debug)]
pub struct SimplexNoise {
    seed: u32,
    scale: f32,
    strength: f32,
}

impl SimplexNoise {
    pub fn new(seed: u32, scale: f32, strength: f32) -> Self {
        Self {
            seed,
            scale,
            strength,
        }
    }

    pub fn sample(&self, point: Vec4) -> f32 {
        scaled_value(simplex4(self.seed, point * self.scale), self.strength)
    }
}

#[cfg(test)]
mod tests {
    use glam::{Vec3, Vec4};

    use super::SimplexNoise;

    #[test]
    fn simplex_noise_samples_stay_in_range() {
        let noise = SimplexNoise::new(7, 2.0, 1.0);

        assert!((-1.0..=1.0).contains(&noise.sample(Vec4::new(0.3, -0.1, 0.7, 0.5))));
    }

    #[test]
    fn simplex_noise_can_vary_over_time() {
        let noise = SimplexNoise::new(7, 2.0, 1.0);
        let point = Vec3::new(0.3, -0.1, 0.7);

        assert_ne!(
            noise.sample(point.extend(0.0)),
            noise.sample(point.extend(0.5)),
        );
    }
}
