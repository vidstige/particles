use crate::field::Field;

pub fn solve_poisson_jacobi(rhs: &Field<f32>, iterations: usize) -> Field<f32> {
    let mut x = rhs.new_like(0.0);
    let mut next_x = rhs.new_like(0.0);
    let cell_size = rhs.cell_size();
    let inverse_dx2 = 1.0 / cell_size.x.powi(2);
    let inverse_dy2 = 1.0 / cell_size.y.powi(2);
    let scale = 0.5 / (inverse_dx2 + inverse_dy2);

    for _ in 0..iterations {
        for y in 0..rhs.height() {
            for x_index in 0..rhs.width() {
                let x_index = x_index as isize;
                let y_index = y as isize;
                let index = rhs.index(x_index, y_index);
                let left = x.get_wrapped(x_index - 1, y_index);
                let right = x.get_wrapped(x_index + 1, y_index);
                let down = x.get_wrapped(x_index, y_index - 1);
                let up = x.get_wrapped(x_index, y_index + 1);
                next_x.set_index(
                    index,
                    ((left + right) * inverse_dx2 + (down + up) * inverse_dy2 - rhs.get(index))
                        * scale,
                );
            }
        }
        std::mem::swap(&mut x, &mut next_x);
    }

    x
}
