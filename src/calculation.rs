pub(crate) struct OptimalPositions {
    pub(crate) bridge_position: f32,
    pub(crate) neck_position: f32,
}

pub(crate) fn get_anti_nodes_for_harmonic(length: f32, harmonic: u8) -> Vec<f32> {
    let segment_length = length / harmonic as f32;

    (0..harmonic)
        .map(|i| (i as f32 + 0.5) * segment_length)
        .collect()
}

pub(crate) fn find_optimal_pickup_positions(
    length: f32,
    weights: &[f32; 6],
    search_limit: usize,
) -> OptimalPositions {
    // Search in the first 50% of string length from bridge (typical pickup placement)
    // let search_limit = (length * 0.5) as usize;
    let resolution = 1000;

    // Calculate score at each position
    let scores: Vec<(f32, f32)> = (0..=search_limit)
        .map(|i| {
            let pos = (i as f32 / resolution as f32) * length;
            let score: f32 = (2..=7_u8)
                .zip(weights.iter())
                .map(|(harmonic, &weight)| {
                    let min_dist = get_anti_nodes_for_harmonic(length, harmonic)
                        .into_iter()
                        .map(|anti_node| (pos - anti_node).abs())
                        .min_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap();

                    // Use same sine wave falloff as heat map
                    let wavelength = length / (harmonic as f32 * 2.0);
                    let normalized_dist = (min_dist / wavelength).min(1.0);
                    let falloff = (normalized_dist * std::f32::consts::PI / 2.0).cos();

                    weight * falloff
                })
                .sum();

            (pos, score)
        })
        .collect();

    // Find the first peak (bridge pickup) - global maximum
    let (bridge_idx, &(bridge_pos, bridge_score)) = scores
        .iter()
        .enumerate()
        .max_by(|(_, (_, a)), (_, (_, b))| a.partial_cmp(b).unwrap())
        .unwrap();

    // Exclude region around the bridge peak
    // We need to expand outward from the peak until values stop decreasing
    // Use a minimum exclusion window as well (e.g., 10% of string length)
    let min_exclusion_distance = length * 0.1; // Pickups should be at least 10% of length apart

    // Find exclusion range by walking away from peak until score increases again
    let mut left_bound = bridge_idx;
    let mut right_bound = bridge_idx;

    // Walk left
    let mut prev_score = bridge_score;
    for i in (0..bridge_idx).rev() {
        let curr_score = scores[i].1;
        if curr_score > prev_score {
            // Score started increasing again, stop here
            break;
        }
        if (bridge_pos - scores[i].0).abs() >= min_exclusion_distance {
            // We've gone far enough to consider this outside the peak region
            if curr_score < bridge_score * 0.5 {
                // If we've dropped below 50% of peak, we're definitely clear
                break;
            }
        }
        left_bound = i;
        prev_score = curr_score;
    }

    // Walk right
    prev_score = bridge_score;
    for i in (bridge_idx + 1)..scores.len() {
        let curr_score = scores[i].1;
        if curr_score > prev_score {
            // Score started increasing again, stop here
            break;
        }
        if (scores[i].0 - bridge_pos).abs() >= min_exclusion_distance {
            // We've gone far enough to consider this outside the peak region
            if curr_score < bridge_score * 0.5 {
                // If we've dropped below 50% of peak, we're definitely clear
                break;
            }
        }
        right_bound = i;
        prev_score = curr_score;
    }

    // Find the second peak (neck pickup) from the remaining data
    // Search in regions [0..left_bound] and [right_bound..end]
    let neck_pos = scores[0..left_bound]
        .iter()
        .chain(scores[right_bound..].iter())
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .map(|&(pos, _)| pos)
        .unwrap_or(length * 0.3); // Fallback to 30% if no second peak found

    OptimalPositions {
        bridge_position: bridge_pos,
        neck_position: neck_pos,
    }
}
