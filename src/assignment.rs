use crate::cloud::Cloud;

pub fn match_clouds(source: &Cloud, target: &Cloud) -> Cloud {
    assert_eq!(source.positions.len(), target.positions.len());

    let n = source.positions.len();
    let mut edges = source
        .positions
        .iter()
        .enumerate()
        .flat_map(|(source_index, from)| {
            target.positions.iter().enumerate().map(move |(target_index, to)| {
                (from.distance_squared(*to), source_index, target_index)
            })
        })
        .collect::<Vec<_>>();
    edges.sort_unstable_by(|left, right| {
        left.0
            .total_cmp(&right.0)
            .then_with(|| left.1.cmp(&right.1))
            .then_with(|| left.2.cmp(&right.2))
    });

    let mut matched_sources = vec![false; n];
    let mut matched_targets = vec![false; n];
    let mut pairs = vec![0; n];
    let mut count = 0;

    for (_, source_index, target_index) in edges {
        if matched_sources[source_index] || matched_targets[target_index] {
            continue;
        }
        matched_sources[source_index] = true;
        matched_targets[target_index] = true;
        pairs[source_index] = target_index;
        count += 1;
        if count == n {
            break;
        }
    }
    assert_eq!(count, n);

    let positions = pairs
        .into_iter()
        .map(|index| target.positions[index])
        .collect();

    Cloud { positions }
}

#[cfg(test)]
mod tests {
    use glam::Vec3;

    use super::match_clouds;
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
}
