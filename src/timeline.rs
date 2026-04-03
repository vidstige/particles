use glam::{Mat4, Vec2, Vec3};

use crate::{
    distribution::{collect, Uniform3},
    gerstner::{displaced_position, surface_grid, GerstnerWave},
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

fn simplex_offset(field: &[SimplexNoise; 3], point: Vec3, w: f32) -> Vec3 {
    Vec3::new(
        field[0].sample(point.extend(w)),
        field[1].sample(point.extend(w)),
        field[2].sample(point.extend(w)),
    )
}

fn simplex_positions(rest_positions: &[Vec3], field: &[SimplexNoise; 3], w: f32) -> Vec<Vec3> {
    rest_positions
        .iter()
        .map(|rest_position| *rest_position + simplex_offset(field, *rest_position, w) * 0.45)
        .collect()
}

fn sample_path(points: &[Vec3], t: f32) -> Vec3 {
    let scaled = t * (points.len() - 1) as f32;
    let index = scaled.floor() as usize;
    let next = (index + 1).min(points.len() - 1);
    points[index].lerp(points[next], scaled.fract())
}

fn orbit_view(t: f32) -> Mat4 {
    let angular_velocity = std::f32::consts::TAU / 48.0;
    let radius = 4.0;
    let height = 1.5;
    let angle = t * angular_velocity;
    let eye = Vec3::new(radius * angle.cos(), height, radius * angle.sin());
    Mat4::look_at_rh(eye, Vec3::ZERO, Vec3::Y)
}

fn glide_view(t: f32) -> Mat4 {
    let phase = (t / 4.0).fract();
    let eye = sample_path(
        &[
            Vec3::new(-1.0, 1.15, -1.7),
            Vec3::new(0.3, 1.05, -1.45),
            Vec3::new(1.0, 1.1, -1.05),
            Vec3::new(-0.6, 1.2, -0.9),
        ],
        phase,
    );
    let center = sample_path(
        &[
            Vec3::new(-0.2, 0.0, 0.2),
            Vec3::new(0.2, 0.0, 0.35),
            Vec3::new(0.55, 0.0, 0.15),
            Vec3::new(0.0, 0.0, 0.05),
        ],
        phase,
    );
    Mat4::look_at_rh(eye, center, Vec3::Y)
}

fn simplex_view() -> Mat4 {
    let eye = Vec3::new(0.35, 0.2, 0.9);
    let center = Vec3::new(-0.15, 0.1, -0.1);
    Mat4::look_at_rh(eye, center, Vec3::Y)
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
        if t < 36.0 {
            self.gerstner_rest_positions
                .iter()
                .map(|rest_position| displaced_position(*rest_position, &self.gerstner_waves, t))
                .collect()
        } else {
            let simplex_speed = 1.0;
            let time = t - 36.0;
            let w = time * simplex_speed;
            simplex_positions(&self.simplex_rest_positions, &self.simplex_field, w)
        }
    }

    pub fn view(&self, t: f32) -> Mat4 {
        if t >= 36.0 {
            return simplex_view();
        }

        if (t / 4.0).floor() as i32 % 2 == 0 {
            orbit_view(t)
        } else {
            glide_view(t)
        }
    }
}
