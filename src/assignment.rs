use crate::cloud::Cloud;

pub fn match_clouds(source: &Cloud, target: &Cloud) -> Cloud {
    assert_eq!(source.positions.len(), target.positions.len());

    let n = source.positions.len();
    let costs = source
        .positions
        .iter()
        .flat_map(|from| {
            target.positions.iter().map(|to| {
                ((from.distance_squared(*to) * 1_000_000.0).round() as u64).max(1)
            })
        })
        .collect::<Vec<_>>();

    let assignment = hungarian::minimize(&costs, n, n);
    let positions = assignment
        .into_iter()
        .map(|column| target.positions[column.expect("square cost matrix")])
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
