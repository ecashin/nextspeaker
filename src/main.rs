use std::{collections::HashMap, fs, path::Path};

use anyhow::{anyhow, bail, Context, Result};
use clap::Parser;
use log::info;
use regex::Regex;

use nextspeaker::{choose, Args};

fn non_blanks_nor_comments(path: &Path) -> Result<Vec<String>> {
    let content = fs::read_to_string(path).with_context(|| format!("reading {path:?}"))?;
    Ok(content
        .lines()
        .filter(|i| !i.is_empty() && !i.starts_with('#'))
        .map(|s| s.to_string())
        .collect())
}

fn maybe_trim_history(history: Vec<String>, do_it: bool) -> Result<Vec<String>> {
    if !do_it {
        return Ok(history);
    }
    let re = Regex::new(r"^\S+\s+(.*)$").unwrap();
    Ok(history
        .into_iter()
        .map(|line| match re.captures(&line) {
            None => bail!("cannot trim history from line: {}", &line),
            Some(groups) => Ok(groups[1].to_string()),
        })
        .collect::<Result<Vec<_>>>()?)
}

fn main() -> Result<()> {
    simple_logger::init_with_env().context("initializing logger")?;
    let args = Args::parse();

    let participants = non_blanks_nor_comments(&args.participants)?;
    let history = maybe_trim_history(
        if let Some(hist_path) = &args.history {
            non_blanks_nor_comments(hist_path)?
        } else {
            vec![]
        },
        args.history_trim,
    )
    .context("processing history")?;

    if participants.is_empty() {
        return Err(anyhow!("participant list is empty"));
    }
    if let Some(n_simulations) = args.n_simulations {
        let mut counts: HashMap<_, _> = HashMap::new();
        for _ in 0..n_simulations {
            let selection = choose(&participants, &history, args.history_halflife)
                .context("choosing participant")?;
            counts
                .entry(selection)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }
        let plen = participants
            .iter()
            .map(|p| p.chars().map(|_| 1).sum::<usize>())
            .max()
            .ok_or_else(|| anyhow!("cannot get maximum-length participant name"))?;
        for p in participants {
            let count = counts.get(&p);
            println!(
                "{:>width$}: {}",
                p,
                if let Some(n) = count { *n } else { 0 },
                width = plen + 1
            );
        }
    } else {
        let selection = choose(&participants, &history, args.history_halflife)
            .context("choosing participant")?;
        info!("selection:{}", &selection);
        println!("{}", selection);
    }
    Ok(())
}
