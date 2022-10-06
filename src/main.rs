use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context, Result};
use clap::Parser;

use selection::choose;

mod selection;

#[derive(Parser, Debug)]
struct Args {
    participants: PathBuf,

    #[arg(long)]
    history: Option<PathBuf>,

    #[arg(long)]
    verbosity: Option<usize>,
}

fn non_blanks(path: &Path) -> Result<Vec<String>> {
    let content = fs::read_to_string(path).with_context(|| format!("reading {path:?}"))?;
    Ok(content
        .lines()
        .into_iter()
        .filter(|i| !i.is_empty())
        .map(|s| s.to_string())
        .collect())
}

fn main() -> Result<()> {
    let args = Args::parse();

    let participants = non_blanks(&args.participants)?;
    let history = if let Some(hist_path) = args.history {
        non_blanks(&hist_path)?
    } else {
        vec![]
    };
    if participants.is_empty() {
        Err(anyhow!("participant list is empty"))
    } else {
        let selection =
            choose(&participants, &history, args.verbosity).context("choosing participant")?;
        println!("{selection}");
        Ok(())
    }
}
