use anyhow::{anyhow, Context, Result};
use rand::distributions::WeightedIndex;
use rand_distr::{Beta, Distribution};

pub fn choose(
    participants: &[String],
    history: &[String],
    verbosity: Option<usize>,
) -> Result<String> {
    let rng = &mut rand::thread_rng();
    let weights = if history.is_empty() {
        vec![1.0; participants.len()]
    } else {
        let recent = (history.len() as f64).log(2.0) as usize;
        participants
            .iter()
            .map(|name| {
                let n_past = history.iter().filter(|i| *i == name).count();
                if history
                    .iter()
                    .skip(history.len() - recent)
                    .any(|i| i == name)
                {
                    0.0
                } else {
                    let dist = Beta::new((1 + n_past) as f64, 1_f64).unwrap();
                    dist.sample(rng)
                }
            })
            .collect()
    };
    if let Some(v) = verbosity {
        if v > 0 {
            let info = weights
                .iter()
                .zip(participants.iter())
                .map(|(w, name)| format!("{name}:{w:.2}"))
                .collect::<Vec<_>>();
            println!("{info:?}");
        }
    }
    let dist = WeightedIndex::new(&weights).context("creating weighted index")?;
    Ok(participants
        .get(dist.sample(rng))
        .ok_or_else(|| anyhow!("weighted index sample is not in bounds"))?
        .to_string())
}
