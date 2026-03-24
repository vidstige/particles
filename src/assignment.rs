use crate::cloud::Cloud;

const DEFAULT_EPSILON: f32 = 0.01;

fn cost_matrix(source: &Cloud, target: &Cloud) -> Vec<f32> {
    source
        .positions
        .iter()
        .flat_map(|from| target.positions.iter().map(move |to| from.distance_squared(*to)))
        .collect()
}

fn best_objects(costs: &[f32], prices: &[f32], bidder: usize, n: usize) -> (usize, f32, f32) {
    let row = &costs[bidder * n..(bidder + 1) * n];
    let mut best_object = 0;
    let mut best_cost = f32::INFINITY;
    let mut second_best_cost = f32::INFINITY;

    for (object, cost) in row.iter().enumerate() {
        let total_cost = *cost + prices[object];
        if total_cost < best_cost {
            second_best_cost = best_cost;
            best_cost = total_cost;
            best_object = object;
        } else if total_cost < second_best_cost {
            second_best_cost = total_cost;
        }
    }

    (best_object, best_cost, second_best_cost)
}

fn auction_assignment(costs: &[f32], n: usize, epsilon: f32) -> Vec<usize> {
    if n == 0 {
        return Vec::new();
    }

    assert!(epsilon.is_finite() && epsilon > 0.0);

    let mut prices = vec![0.0; n];
    let mut owners = vec![None; n];
    let mut assignments = vec![None; n];
    let mut unassigned = (0..n).rev().collect::<Vec<_>>();

    while let Some(bidder) = unassigned.pop() {
        let (object, best_cost, second_best_cost) = best_objects(costs, &prices, bidder, n);
        let raise = if second_best_cost.is_finite() {
            second_best_cost - best_cost + epsilon
        } else {
            epsilon
        };
        prices[object] += raise;

        if let Some(previous_bidder) = owners[object].replace(bidder) {
            assignments[previous_bidder] = None;
            unassigned.push(previous_bidder);
        }
        assignments[bidder] = Some(object);
    }

    assignments.into_iter().map(|assignment| assignment.unwrap()).collect()
}

pub fn match_clouds_with_epsilon(source: &Cloud, target: &Cloud, epsilon: f32) -> Cloud {
    assert_eq!(source.positions.len(), target.positions.len());

    let assignment = auction_assignment(&cost_matrix(source, target), source.positions.len(), epsilon);
    let positions = assignment
        .into_iter()
        .map(|index| target.positions[index])
        .collect();

    Cloud { positions }
}

pub fn match_clouds(source: &Cloud, target: &Cloud) -> Cloud {
    match_clouds_with_epsilon(source, target, DEFAULT_EPSILON)
}

#[cfg(test)]
mod tests {
    use glam::Vec3;

    use super::{match_clouds, match_clouds_with_epsilon};
    use crate::cloud::Cloud;

    #[test]
    fn reorders_target_to_minimum_cost_bijection() {
        let source = Cloud {
            positions: vec![
                Vec3::new(-1.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(1.0, 0.0, 0.0),
            ],
        };
        let target = Cloud {
            positions: vec![
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(-1.0, 0.0, 0.0),
            ],
        };

        let matched = match_clouds(&source, &target);

        assert_eq!(matched.positions, source.positions);
    }

    #[test]
    fn epsilon_variant_preserves_bijection() {
        let source = Cloud {
            positions: vec![
                Vec3::new(-1.0, 0.0, 0.0),
                Vec3::new(0.0, -1.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(1.0, 0.0, 0.0),
            ],
        };
        let target = Cloud {
            positions: vec![
                Vec3::new(1.1, 0.1, 0.0),
                Vec3::new(-1.1, -0.1, 0.0),
                Vec3::new(0.1, 1.1, 0.0),
                Vec3::new(-0.1, -1.1, 0.0),
            ],
        };

        let matched = match_clouds_with_epsilon(&source, &target, 0.1);
        let mut matched_positions = matched.positions.clone();
        let mut target_positions = target.positions.clone();

        matched_positions.sort_unstable_by(|left, right| left.x.total_cmp(&right.x).then(left.y.total_cmp(&right.y)));
        target_positions.sort_unstable_by(|left, right| left.x.total_cmp(&right.x).then(left.y.total_cmp(&right.y)));

        assert_eq!(matched_positions, target_positions);
    }
}
