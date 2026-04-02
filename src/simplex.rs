use glam::Vec2;

pub struct SimplexNoise {
    pub scale: f32,
    pub strength: f32,
}

impl SimplexNoise {
    pub fn sample(&self, point: &Vec2) -> f32 {
        const F2: f32 = 0.366_025_42;
        const G2: f32 = 0.211_324_87;

        let point = point * self.scale;
        let skew = (point.x + point.y) * F2;
        let cell = (point + Vec2::new(skew, skew)).map(|x| x.floor());
        let unskew = (cell.x + cell.y) * G2;
        let origin = cell - Vec2::new(unskew, unskew);
        let local0 = point - origin;

        let offset = if local0.x > local0.y {
            Vec2::new(1.0, 0.0)
        } else {
            Vec2::new(0.0, 1.0)
        };

        let local1 = local0 - offset + Vec2::new(G2, G2);
        let local2 = local0 - Vec2::new(1.0, 1.0) + Vec2::new(2.0 * G2, 2.0 * G2);

        let n0 = contribution(&(cell), local0);
        let n1 = contribution(&(cell + offset), local1);
        let n2 = contribution(&(cell + Vec2::new(1.0, 1.0)), local2);

        ((70.0 * (n0 + n1 + n2)) * self.strength).clamp(-1.0, 1.0)
    }
}

fn contribution(cell: &Vec2, local: Vec2) -> f32 {
    let t = 0.5 - local.dot(local);
    if t < 0.0 {
        0.0
    } else {
        let gradient = gradient(cell);
        let weight = t * t;
        weight * weight * gradient.dot(local)
    }
}

fn gradient(cell: &Vec2) -> Vec2 {
    let angle = hash(cell.x as i32, cell.y as i32) * std::f32::consts::TAU;
    Vec2::new(angle.cos(), angle.sin())
}

fn hash(x: i32, y: i32) -> f32 {
    let mut n = (x as u32).wrapping_mul(0x27d4_eb2d) ^ (y as u32).wrapping_mul(0x1656_67b1);
    n ^= n >> 15;
    n = n.wrapping_mul(0x85eb_ca6b);
    n ^= n >> 13;
    (n as f32) / (u32::MAX as f32)
}
