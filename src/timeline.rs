use glam::{Vec2, Vec3};

use crate::{
    distribution::{collect, Uniform3},
    gerstner::{displaced_positions, surface_grid, GerstnerWave},
    rng::Rng,
    simplex::SimplexNoise,
};

fn water_waves() -> [GerstnerWave; 5] {
    [
        GerstnerWave::new(Vec2::new(1.0, 0.1), 0.11, 2.8, 0.55, 0.75, 0.0),
        GerstnerWave::new(Vec2::new(0.2, 1.0), 0.08, 1.9, 0.8, 0.7, 0.8),
        GerstnerWave::new(Vec2::new(-0.9, 0.4), 0.05, 1.1, 1.1, 0.55, 1.7),
        GerstnerWave::new(Vec2::new(0.7, -0.6), 0.04, 0.75, 1.4, 0.45, 2.2),
        GerstnerWave::new(Vec2::new(-0.3, -1.0), 0.03, 0.5, 1.8, 0.35, 0.5),
    ]
}

fn simplex_field() -> [SimplexNoise; 3] {
    [
        SimplexNoise::new(0x1f2e_3d4c, 1.4, 1.0),
        SimplexNoise::new(0x2a39_4857, 1.4, 1.0),
        SimplexNoise::new(0x6574_8392, 1.4, 1.0),
    ]
}

fn simplex_offset(field: &[SimplexNoise; 3], point: Vec3, time: f32) -> Vec3 {
    Vec3::new(
        field[0].sample(point.extend(time)),
        field[1].sample(point.extend(time + 7.0)),
        field[2].sample(point.extend(time + 13.0)),
    )
}

fn simplex_positions(rest_positions: &[Vec3], field: &[SimplexNoise; 3], time: f32) -> Vec<Vec3> {
    rest_positions
        .iter()
        .map(|rest_position| *rest_position + simplex_offset(field, *rest_position, time) * 0.45)
        .collect()
}

pub struct Timeline {
    gerstner_rest_positions: Vec<Vec3>,
    gerstner_waves: [GerstnerWave; 5],
    simplex_rest_positions: Vec<Vec3>,
    simplex_field: [SimplexNoise; 3],
}

impl Timeline {
    pub fn new() -> Self {
        let gerstner_rest_positions = surface_grid(64, 64, Vec2::new(8.0, 8.0));
        let mut rng = Rng::new(0x1234_5678);
        let simplex_rest_positions =
            collect(&mut Uniform3::new(), gerstner_rest_positions.len(), &mut rng);

        Self {
            gerstner_rest_positions,
            gerstner_waves: water_waves(),
            simplex_rest_positions,
            simplex_field: simplex_field(),
        }
    }

    pub fn particle_count(&self) -> usize {
        self.gerstner_rest_positions.len()
    }

    pub fn particles(&self, t: f32) -> Vec<Vec3> {
        if t < 5.0 {
            displaced_positions(&self.gerstner_rest_positions, &self.gerstner_waves, t)
        } else {
            simplex_positions(&self.simplex_rest_positions, &self.simplex_field, t - 5.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{simplex_positions, Timeline};
    use crate::gerstner::displaced_positions;

    #[test]
    fn timeline_uses_gerstner_before_five_seconds() {
        let timeline = Timeline::new();

        assert_eq!(
            timeline.particles(1.5),
            displaced_positions(
                &timeline.gerstner_rest_positions,
                &timeline.gerstner_waves,
                1.5,
            ),
        );
    }

    #[test]
    fn timeline_uses_simplex_from_five_seconds() {
        let timeline = Timeline::new();

        assert_eq!(
            timeline.particles(5.25),
            simplex_positions(
                &timeline.simplex_rest_positions,
                &timeline.simplex_field,
                0.25,
            ),
        );
    }
}
