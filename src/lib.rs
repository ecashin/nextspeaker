use std::{cmp::min, path::PathBuf};

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use log::{debug, info};
use rand::distributions::WeightedIndex;
use rand_distr::{Beta, Distribution};

pub const DEFAULT_HALFLIFE: f64 = 10.0;

#[derive(Parser, Debug)]
pub struct Args {
    pub participants: PathBuf,
    #[arg(long)]
    pub history: Option<PathBuf>,
    #[arg(long, default_value_t = DEFAULT_HALFLIFE)]
    pub history_halflife: f64,
    #[arg(long)]
    pub n_simulations: Option<usize>,
}

fn exponentially_weighted_decay(half_life: f64, time: f64) -> f64 {
    0.5_f64.powf(time / half_life)
}

fn n_recent_for_history_and_participants(n_history: usize, n_participants: usize) -> usize {
    let recent = (n_history as f64).log(2.0) as usize;
    min(recent, n_participants / 2)
}

pub fn choose(
    participants: &[String],
    history: &[String],
    history_halflife: f64,
) -> Result<String> {
    debug!("history:{history:?}");
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
                exponentially_weighted_decay(history_halflife, t)
            })
            .rev()
            .collect::<Vec<_>>();
        let recent = n_recent_for_history_and_participants(history.len(), participants.len());
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
        let mut recent_participants = vec![];
        let mut weights: Vec<_> = participants
            .iter()
            .zip(history_weights.iter())
            .enumerate()
            .map(|(i, (name, &weight_past))| {
                if history
                    .iter()
                    .skip(history.len() - recent)
                    .any(|p| p == name)
                {
                    recent_participants.push(i);
                }
                // Beta distribution will lean toward zero weight
                // the more a participant has been previously selected.
                debug!("participant:{name} history weight:{weight_past}");
                let dist = Beta::new(1_f64, 1_f64 + weight_past).unwrap();
                dist.sample(rng)
            })
            .collect();
        info!("recent participants:{recent_participants:?}");
        // Exclude recently selected participants unless everyone's recent
        if recent_participants.len() < participants.len() {
            for i in recent_participants {
                weights[i] = 0.0;
            }
        }
        weights
    };
    let weight_info = weights
        .iter()
        .zip(participants.iter())
        .map(|(w, name)| format!("{name}:{w:.2}"))
        .collect::<Vec<_>>();
    info!("participant selection weights:{weight_info:?}");
    let dist = WeightedIndex::new(&weights).context("creating weighted index")?;
    Ok(participants
        .get(dist.sample(rng))
        .ok_or_else(|| anyhow!("weighted index sample is not in bounds"))?
        .to_string())
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use log::debug;
    use rand::thread_rng;

    use super::*;

    #[derive(Debug)]
    struct HistSlice {
        duration: usize,
        favorites: Vec<(usize, f64)>,
    }

    impl HistSlice {
        fn weights(&self, n_participants: usize) -> Vec<f64> {
            let Self { favorites, .. } = self;
            let favorites: HashMap<_, f64> = favorites.iter().copied().collect();
            (0..n_participants)
                .map(|i| {
                    let favor = favorites.get(&i);
                    if let Some(favor) = favor {
                        *favor
                    } else {
                        1.0 / (n_participants as f64)
                    }
                })
                .collect::<Vec<_>>()
        }
    }

    #[derive(Debug)]
    struct Scenario {
        pub participants: Vec<String>,
        pub history: Vec<String>,
    }

    const UNIVERSE: &str = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

    impl Scenario {
        fn new(n_participants: usize, history_slices: &[HistSlice], non_recent: &[usize]) -> Self {
            debug!("new scenario for history_slices:{history_slices:?}");
            assert!(n_participants <= UNIVERSE.len());
            let mut rng = thread_rng();
            let participants: Vec<_> = UNIVERSE
                .chars()
                .take(n_participants)
                .map(|c| c.to_string())
                .collect();
            let mut history: Vec<_> = history_slices
                .iter()
                .map(|hist_slice| {
                    let weights = hist_slice.weights(n_participants);
                    (0..hist_slice.duration)
                        .map(|_| {
                            let dist = WeightedIndex::new(&weights).unwrap();
                            let choice = dist.sample(&mut rng);
                            debug!(
                                "chose {} using participant weights:{weights:?}",
                                participants[choice]
                            );
                            participants[choice].clone()
                        })
                        .collect::<Vec<_>>()
                })
                .flatten()
                .collect::<Vec<_>>();
            let n_recent = n_recent_for_history_and_participants(history.len(), n_participants);
            loop {
                let mut n_changed = 0;
                for i in (history.len() - n_recent)..history.len() {
                    for to_omit in non_recent {
                        if history[i] == participants[*to_omit] {
                            let weights =
                                history_slices[history_slices.len() - 1].weights(n_participants);
                            let choice = WeightedIndex::new(&weights).unwrap().sample(&mut rng);
                            debug!(
                                "replacing {} in recent participants ({:?}) with {}",
                                participants[*to_omit],
                                &history[(history.len() - n_recent)..history.len()],
                                participants[choice]
                            );
                            history[i] = participants[choice].clone();
                            n_changed += 1;
                        }
                    }
                }
                if n_changed == 0 {
                    break;
                }
            }
            debug!("history:{history:?}");
            Self {
                participants,
                history,
            }
        }
    }

    const N_REPS: usize = 1000;

    fn init() -> Result<()> {
        Ok(simple_logger::init_with_env()?)
    }

    #[test]
    fn test_forgetting() -> Result<()> {
        init()?;
        let s = Scenario::new(
            10,
            &[
                HistSlice {
                    duration: 1000,
                    favorites: vec![(0, 100.0)],
                },
                HistSlice {
                    duration: 100,
                    favorites: vec![(1, 10.0)],
                },
                HistSlice {
                    duration: 100,
                    favorites: vec![(2, 10.0)],
                },
                HistSlice {
                    duration: 100,
                    favorites: vec![],
                },
            ],
            &[0, 1, 2],
        );
        debug!("scenario:\n{s:?}");
        let halflife_vals = [1000.0, 1.0];
        let mut selected = halflife_vals
            .iter()
            .map(|_| HashMap::new())
            .collect::<Vec<_>>();
        for _ in 0..N_REPS {
            info!("halflife:{}", halflife_vals[0]);
            let selection = choose(&s.participants, &s.history, halflife_vals[0])?;
            *selected[0].entry(selection).or_insert(0_usize) += 1;

            info!("halflife:{}", halflife_vals[1]);
            let selection = choose(&s.participants, &s.history, halflife_vals[1])?;
            *selected[1].entry(selection).or_insert(0_usize) += 1;
        }
        debug!("selected:{selected:?}");
        let counts = [0, 1]
            .into_iter()
            .map(|i| {
                let count = *selected[i].entry(s.participants[0].clone()).or_insert(0);
                info!("0 count with half-life:{} is {count}", halflife_vals[i]);
                count
            })
            .collect::<Vec<_>>();
        info!("counts:{counts:?}");
        assert!(counts[0] < counts[1]);
        Ok(())
    }

    #[test]
    fn test_recent() -> Result<()> {
        let args = &Args::dummy();
        let participants = "abcdefghijklmnopqrstuvwxyz"
            .chars()
            .map(|c| c.to_string())
            .collect::<Vec<_>>();
        let history = participants.clone();
        for _ in 0..N_REPS {
            let choice = choose(&participants, &history, args.history_halflife)?;
            for name in ["x", "y", "z"] {
                assert_ne!(choice, name);
            }
        }
        Ok(())
    }

    impl Args {
        pub fn dummy() -> Self {
            Self {
                participants: PathBuf::from("dummy"),
                history: Some(PathBuf::from("dummy-history")),
                history_halflife: 10.0,
                n_simulations: None,
            }
        }
        pub fn dummy_with_halflife(halflife: f64) -> Self {
            let mut args = Self::dummy();
            args.history_halflife = halflife;
            args
        }
    }
}
