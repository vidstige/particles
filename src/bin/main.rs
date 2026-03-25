use std::io::{self, Write};

use glam::{Mat4, Vec3};
use particles::{
    assignment::auction_assignment,
    distribution::{
        cube, gaussian_sphere, grid_3d, gyroid, icosahedron, lissajous, sphere, tetrahedron,
        torus_surface, uniform_cube,
    },
    render::render_cloud,
    resolution::Resolution,
    rng::Rng,
};

#[derive(Debug)]
struct Cloud {
    positions: Vec<Vec3>,
}

fn cost_matrix(source: &[Vec3], target: &[Vec3]) -> Vec<f32> {
    source
        .iter()
        .flat_map(|from| target.iter().map(move |to| from.distance_squared(*to)))
        .collect()
}

fn interpolate_cloud(source: &Cloud, target: &Cloud, t: f32) -> Cloud {
    Cloud {
        positions: source
            .positions
            .iter()
            .zip(&target.positions)
            .map(|(from, to)| from.lerp(*to, t))
            .collect(),
    }
}

fn max_radius(clouds: &[Cloud]) -> f32 {
    clouds
        .iter()
        .flat_map(|cloud| cloud.positions.iter())
        .map(|point| point.length())
        .fold(0.0, f32::max)
}

fn match_positions(source: &[Vec3], target: &[Vec3], epsilon: f32) -> Vec<Vec3> {
    assert_eq!(source.len(), target.len());

    let assignment = auction_assignment(&cost_matrix(source, target), source.len(), epsilon);
    assignment.into_iter().map(|index| target[index]).collect()
}

fn main() -> io::Result<()> {
    let mut output = io::stdout().lock();
    let resolution = Resolution::new(720, 720);
    let mut rng = Rng::new(0x1234_5678);
    let point_count = 512;
    let epsilon = 0.01;
    let frame_count = 32;
    let mut clouds = vec![
        Cloud {
            positions: uniform_cube(point_count, &mut rng),
        },
        Cloud {
            positions: cube(point_count, 0.9, &mut rng),
        },
        Cloud {
            positions: grid_3d(point_count, Vec3::splat(0.18)),
        },
        Cloud {
            positions: sphere(point_count, 0.95, &mut rng),
        },
        Cloud {
            positions: tetrahedron(point_count, 0.95, &mut rng),
        },
        Cloud {
            positions: torus_surface(point_count, 0.75, 0.25, &mut rng),
        },
        Cloud {
            positions: icosahedron(point_count, 0.95, &mut rng),
        },
        Cloud {
            positions: lissajous(point_count, 0.9),
        },
        Cloud {
            positions: gyroid(point_count, 1.1, 0.08, &mut rng),
        },
        Cloud {
            positions: gaussian_sphere(point_count, &mut rng),
        },
    ];

    for index in 1..clouds.len() {
        clouds[index].positions = match_positions(
            &clouds[index - 1].positions,
            &clouds[index].positions,
            epsilon,
        );
    }

    let radius = max_radius(&clouds).max(1.0);
    let eye = Vec3::new(0.0, 0.0, radius * 3.5);
    let view = Mat4::look_at_rh(eye, Vec3::ZERO, Vec3::Y);
    let projection = Mat4::perspective_rh_gl(
        50.0_f32.to_radians(),
        resolution.aspect_ratio(),
        0.1,
        eye.z + radius * 2.0,
    );

    for (index, pair) in clouds.windows(2).enumerate() {
        let source = &pair[0];
        let target = &pair[1];

        for frame in usize::from(index > 0)..frame_count {
            let t = frame as f32 / (frame_count as f32 - 1.0);
            let cloud = interpolate_cloud(source, target, t);
            let pixmap = render_cloud(&cloud.positions, &resolution, projection, view);
            output.write_all(pixmap.data())?;
            output.flush()?;
        }
    }

    Ok(())
}
