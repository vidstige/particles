use crate::field::Field;

pub fn solve_poisson_gauss_seidel(rhs: &Field<f32>, p: &mut Field<f32>, iterations: usize) {
    let cell_size = rhs.cell_size();
    let inverse_dx2 = 1.0 / cell_size.x.powi(2);
    let inverse_dy2 = 1.0 / cell_size.y.powi(2);
    let scale = 0.5 / (inverse_dx2 + inverse_dy2);

    for _ in 0..iterations {
        for y in 0..rhs.height() {
            for x in 0..rhs.width() {
                let x = x as isize;
                let y = y as isize;
                let index = rhs.index(x, y);
                let left  = p.get_wrapped(x - 1, y);
                let right = p.get_wrapped(x + 1, y);
                let down  = p.get_wrapped(x, y - 1);
                let up    = p.get_wrapped(x, y + 1);
                p.set_index(
                    index,
                    ((left + right) * inverse_dx2 + (down + up) * inverse_dy2 - rhs.get(index))
                        * scale,
                );
            }
        }
    }
}
