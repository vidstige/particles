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

pub fn auction_assignment(costs: &[f32], n: usize, epsilon: f32) -> Vec<usize> {
    if n == 0 {
        return Vec::new();
    }

    assert_eq!(costs.len(), n * n);
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

#[cfg(test)]
mod tests {
    use super::auction_assignment;

    #[test]
    fn auction_assignment_matches_known_3x3_solution() {
        let assignment = auction_assignment(
            &[
                4.0, 1.0, 3.0,
                2.0, 0.0, 5.0,
                3.0, 2.0, 2.0,
            ],
            3,
            0.001,
        );

        assert_eq!(assignment, vec![1, 0, 2]);
    }
}
