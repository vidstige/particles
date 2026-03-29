pub fn lerp<T>(left: T, right: T, t: f32) -> T
where
    T: Copy + std::ops::Add<Output = T> + std::ops::Mul<f32, Output = T>,
{
    left * (1.0 - t) + right * t
}
