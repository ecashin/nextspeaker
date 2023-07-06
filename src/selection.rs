use anyhow::{anyhow, Context, Result};
use rand::distributions::WeightedIndex;
use rand_distr::{Beta, Distribution};

use crate::Args;

fn exponentially_weighted_decay(half_life: f64, time: f64) -> f64 {
    0.5_f64.powf(time / half_life)
}

pub fn choose(participants: &[String], history: &[String], args: &Args) -> Result<String> {
    let verbose = args.verbosity.is_some_and(|x| x > 0);
    let rng = &mut rand::thread_rng();
    let weights = if history.is_empty() {
        vec![1.0; participants.len()]
    } else {
        let decay = history
            .iter()
            .rev()
            .enumerate()
            .map(|(i, _)| {
                let t = i as f64;
                exponentially_weighted_decay(args.history_halflife, t)
            })
            .rev()
            .collect::<Vec<_>>();
        let recent = (history.len() as f64).log(2.0) as usize;
        let history_weights: Vec<_> = participants
            .iter()
            .map(|name| {
                history
                    .iter()
                    .enumerate()
                    .filter_map(
                        |(i, name_i)| {
                            if name_i == name {
                                Some(decay[i])
                            } else {
                                None
                            }
                        },
                    )
                    .sum::<f64>()
            })
            .collect();
        let weights: Vec<_> = participants
            .iter()
            .enumerate()
            .map(|(i, name)| {
                let weight_past = history_weights[i];
                if history
                    .iter()
                    .skip(history.len() - recent)
                    .any(|i| i == name)
                {
                    // Exclude recently selected participants
                    0.0
                } else {
                    // Beta distribution will lean toward zero weight
                    // the more a participant has been previously selected.
                    let dist = Beta::new(1_f64, 1_f64 + weight_past).unwrap();
                    dist.sample(rng)
                }
            })
            .collect();
        // sample based on history if *everyone* was recent
        if weights.iter().sum::<f64>() == 0.0 {
            history_weights
                .iter()
                .enumerate()
                .map(|(i, w)| {
                    let dist = Beta::new(1_f64, 1_f64 + w).unwrap();
                    if verbose {
                        println!("{}: {:?}", participants[i], dist);
                    }
                    dist.sample(rng)
                })
                .collect()
        } else {
            weights
        }
    };
    if verbose {
        // Print out all the weights when user specifies verbosity > 0
        let info = weights
            .iter()
            .zip(participants.iter())
            .map(|(w, name)| format!("{name}:{w:.2}"))
            .collect::<Vec<_>>();
        println!("{info:?}");
    }
    let dist = WeightedIndex::new(&weights).context("creating weighted index")?;
    Ok(participants
        .get(dist.sample(rng))
        .ok_or_else(|| anyhow!("weighted index sample is not in bounds"))?
        .to_string())
}
