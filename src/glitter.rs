use crate::{color::Color, rng::Rng};
use glam::{Mat3, Mat4, Vec3};

#[derive(Clone, Copy, Debug)]
pub struct Glitter {
    pub falloff_power: f32,
    pub tumble_speed: f32,
    pub tumble_axis: Vec3,
    pub precession_axis: Vec3,
    pub precession_speed: f32,
}

fn random_normal(rng: &mut Rng) -> Vec3 {
    Vec3::new(
        rng.next_gaussian(),
        rng.next_gaussian(),
        rng.next_gaussian(),
    )
    .normalize()
}

fn glitter_amount(normal: Vec3, view_direction: Vec3, glitter: Glitter) -> f32 {
    normal
        .dot(view_direction)
        .clamp(0.0, 1.0)
        .powf(glitter.falloff_power)
}

fn lerp(left: f32, right: f32, t: f32) -> f32 {
    left * (1.0 - t) + right * t
}

fn lerp_color(left: Color, right: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color::new(
        lerp(left.red, right.red, t),
        lerp(left.green, right.green, t),
        lerp(left.blue, right.blue, t),
    )
}

pub fn view_direction(view: Mat4) -> Vec3 {
    view.inverse().transform_vector3(-Vec3::Z).normalize()
}

pub fn rotate_normals(normals: &[Vec3], rotation: Mat3) -> Vec<Vec3> {
    normals.iter().map(|normal| rotation * *normal).collect()
}

pub fn tumble_rotation(time: f32, glitter: Glitter) -> Mat3 {
    let tumble_axis = glitter.tumble_axis.normalize_or(Vec3::X);
    let precession_axis = glitter.precession_axis.normalize_or(Vec3::Y);
    let precession = Mat3::from_axis_angle(precession_axis, time * glitter.precession_speed);
    let current_tumble_axis = precession * tumble_axis;
    Mat3::from_axis_angle(current_tumble_axis, time * glitter.tumble_speed)
}

pub fn glitter_normals(rng: &mut Rng, count: usize) -> Vec<Vec3> {
    (0..count).map(|_| random_normal(rng)).collect()
}

pub fn glitter_colors(
    base_color: Color,
    normals: &[Vec3],
    view_direction: Vec3,
    glitter: Glitter,
) -> Vec<Color> {
    let glitter_tint = Color::new(4.0, 4.0, 4.0);
    normals
        .iter()
        .map(|normal| {
            lerp_color(
                base_color,
                glitter_tint,
                glitter_amount(*normal, view_direction, glitter),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{glitter_colors, rotate_normals, tumble_rotation, view_direction, Glitter};
    use crate::color::Color;
    use glam::{Mat3, Mat4, Vec3};

    #[test]
    fn view_direction_matches_look_at_forward_axis() {
        let eye = Vec3::new(2.0, 2.0, 2.0);
        let center = Vec3::ZERO;
        let view = Mat4::look_at_rh(eye, center, Vec3::Y);

        let direction = view_direction(view);

        assert!(direction.abs_diff_eq((center - eye).normalize(), 1e-6));
    }

    #[test]
    fn glitter_colors_brightens_particles_facing_the_camera() {
        let base_color = Color::new(0.25, 0.5, 0.75);
        let view_direction = Vec3::new(0.0, 0.0, -1.0);
        let glitter = Glitter {
            falloff_power: 16.0,
            tumble_speed: 0.0,
            tumble_axis: Vec3::X,
            precession_axis: Vec3::Y,
            precession_speed: 0.0,
        };
        let normals = [view_direction, Vec3::X, -view_direction];

        let colors = glitter_colors(base_color, &normals, view_direction, glitter);

        assert_eq!(colors[0], Color::new(4.0, 4.0, 4.0));
        assert_eq!(colors[1], base_color);
        assert_eq!(colors[2], base_color);
    }

    #[test]
    fn rotate_normals_applies_rotation_to_each_normal() {
        let normals = [Vec3::Y, Vec3::Z];
        let rotation = Mat3::from_rotation_x(std::f32::consts::FRAC_PI_2);
        let rotated = rotate_normals(&normals, rotation);

        assert!(rotated[0].abs_diff_eq(Vec3::Z, 1e-6));
        assert!(rotated[1].abs_diff_eq(-Vec3::Y, 1e-6));
    }

    #[test]
    fn tumble_rotation_changes_normals_over_time() {
        let glitter = Glitter {
            falloff_power: 1.0,
            tumble_speed: 1.0,
            tumble_axis: Vec3::Z,
            precession_axis: Vec3::Y,
            precession_speed: 0.0,
        };
        let normals = [Vec3::X];
        let rotated = rotate_normals(
            &normals,
            tumble_rotation(std::f32::consts::FRAC_PI_2, glitter),
        );

        assert!(rotated[0].abs_diff_eq(Vec3::Y, 1e-6));
    }

    #[test]
    fn tumble_rotation_adds_slow_precession() {
        let glitter = Glitter {
            falloff_power: 1.0,
            tumble_speed: 0.0,
            tumble_axis: Vec3::Z,
            precession_axis: Vec3::X,
            precession_speed: 1.0,
        };
        let normals = [Vec3::Z];
        let rotated = rotate_normals(
            &normals,
            tumble_rotation(std::f32::consts::FRAC_PI_2, glitter),
        );

        assert!(rotated[0].abs_diff_eq(Vec3::Z, 1e-6));
    }

    #[test]
    fn tumble_rotation_uses_precessed_axis_for_spin() {
        let glitter = Glitter {
            falloff_power: 1.0,
            tumble_speed: 1.0,
            tumble_axis: Vec3::Z,
            precession_axis: Vec3::X,
            precession_speed: 1.0,
        };
        let normals = [Vec3::X];
        let rotated = rotate_normals(
            &normals,
            tumble_rotation(std::f32::consts::FRAC_PI_2, glitter),
        );

        assert!(rotated[0].abs_diff_eq(Vec3::Z, 1e-6));
    }

    #[test]
    fn glitter_colors_use_rotated_normals() {
        let base_color = Color::new(0.25, 0.5, 0.75);
        let view_direction = Vec3::new(0.0, 0.0, -1.0);
        let glitter = Glitter {
            falloff_power: 1.0,
            tumble_speed: 1.0,
            tumble_axis: Vec3::X,
            precession_axis: Vec3::Y,
            precession_speed: 0.0,
        };
        let normals = [view_direction];
        let colors_at_start = glitter_colors(base_color, &normals, view_direction, glitter);
        let colors_quarter_turn = glitter_colors(
            base_color,
            &rotate_normals(
                &normals,
                tumble_rotation(std::f32::consts::FRAC_PI_2, glitter),
            ),
            view_direction,
            glitter,
        );

        assert_eq!(colors_at_start[0], Color::new(4.0, 4.0, 4.0));
        assert_eq!(colors_quarter_turn[0], base_color);
    }
}
