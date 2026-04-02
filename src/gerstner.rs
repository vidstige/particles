use glam::{Vec2, Vec3};

#[derive(Clone, Copy, Debug)]
pub struct GerstnerWave {
    pub direction: Vec2,
    pub amplitude: f32,
    pub wavelength: f32,
    pub speed: f32,
    pub steepness: f32,
    pub phase: f32,
}

impl GerstnerWave {
    pub fn new(
        direction: Vec2,
        amplitude: f32,
        wavelength: f32,
        speed: f32,
        steepness: f32,
        phase: f32,
    ) -> Self {
        Self {
            direction,
            amplitude,
            wavelength,
            speed,
            steepness,
            phase,
        }
    }
}

fn wave_offset(wave: GerstnerWave, position: Vec2, time: f32) -> Vec3 {
    let direction = wave.direction.normalize_or_zero();
    let wave_number = std::f32::consts::TAU / wave.wavelength.max(f32::MIN_POSITIVE);
    let phase = wave_number * (direction.dot(position) - wave.speed * time) + wave.phase;
    let horizontal = direction * (wave.steepness * wave.amplitude * phase.cos());
    Vec3::new(horizontal.x, wave.amplitude * phase.sin(), horizontal.y)
}

pub fn displaced_position(rest_position: Vec3, waves: &[GerstnerWave], time: f32) -> Vec3 {
    let position = Vec2::new(rest_position.x, rest_position.z);

    rest_position
        + waves
            .iter()
            .copied()
            .map(|wave| wave_offset(wave, position, time))
            .sum::<Vec3>()
}

pub fn surface_grid(columns: usize, rows: usize, size: Vec2) -> Vec<Vec3> {
    let x_step = size.x / (columns.saturating_sub(1).max(1) as f32);
    let z_step = size.y / (rows.saturating_sub(1).max(1) as f32);
    let x_start = -size.x * 0.5;
    let z_start = -size.y * 0.5;
    let mut positions = Vec::with_capacity(columns * rows);

    for row in 0..rows {
        for column in 0..columns {
            positions.push(Vec3::new(
                x_start + column as f32 * x_step,
                0.0,
                z_start + row as f32 * z_step,
            ));
        }
    }

    positions
}

#[cfg(test)]
mod tests {
    use super::{displaced_position, surface_grid, GerstnerWave};
    use glam::{Vec2, Vec3};

    #[test]
    fn surface_grid_is_centered_on_the_xz_plane() {
        assert_eq!(
            surface_grid(3, 2, Vec2::new(4.0, 2.0)),
            vec![
                Vec3::new(-2.0, 0.0, -1.0),
                Vec3::new(0.0, 0.0, -1.0),
                Vec3::new(2.0, 0.0, -1.0),
                Vec3::new(-2.0, 0.0, 1.0),
                Vec3::new(0.0, 0.0, 1.0),
                Vec3::new(2.0, 0.0, 1.0),
            ]
        );
    }

    #[test]
    fn displaced_position_leaves_rest_position_unchanged_without_waves() {
        let rest_position = Vec3::new(-1.0, 0.5, 2.0);

        assert_eq!(displaced_position(rest_position, &[], 1.5), rest_position);
    }

    #[test]
    fn gerstner_waves_displace_points_sideways_as_well_as_upwards() {
        let wave = GerstnerWave::new(Vec2::X, 0.5, 4.0, 1.0, 0.75, 0.0);

        assert_eq!(
            displaced_position(Vec3::ZERO, &[wave], 0.0),
            Vec3::new(0.375, 0.0, 0.0)
        );
    }

    #[test]
    fn displaced_position_uses_rest_position_instead_of_accumulating_drift() {
        let rest_position = Vec3::new(0.5, 0.0, -0.25);
        let wave = GerstnerWave::new(Vec2::new(1.0, 1.0), 0.4, 3.0, 0.8, 0.6, 0.2);
        let first = displaced_position(rest_position, &[wave], 0.75);
        let second = displaced_position(rest_position, &[wave], 0.75);

        assert_eq!(second, first);
    }
}
