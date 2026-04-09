use crate::field::Field;

pub fn solve_poisson_jacobi(right_hand_side: &Field<f32>, iterations: usize) -> Field<f32> {
    let mut pressure = right_hand_side.new_like(0.0);
    let mut next_pressure = right_hand_side.new_like(0.0);
    let cell_size = right_hand_side.cell_size();
    let inverse_dx2 = 1.0 / cell_size.x.powi(2);
    let inverse_dy2 = 1.0 / cell_size.y.powi(2);
    let scale = 0.5 / (inverse_dx2 + inverse_dy2);

    for _ in 0..iterations {
        for y in 0..right_hand_side.height() {
            for x in 0..right_hand_side.width() {
                let x = x as isize;
                let y = y as isize;
                let index = right_hand_side.index(x, y);
                let left = pressure.get_wrapped(x - 1, y);
                let right = pressure.get_wrapped(x + 1, y);
                let down = pressure.get_wrapped(x, y - 1);
                let up = pressure.get_wrapped(x, y + 1);
                next_pressure.set_index(
                    index,
                    ((left + right) * inverse_dx2 + (down + up) * inverse_dy2
                        - right_hand_side.get(index))
                        * scale,
                );
            }
        }
        std::mem::swap(&mut pressure, &mut next_pressure);
    }

    pressure
}
