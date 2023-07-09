use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use log::info;

use selection::choose;

mod selection;

#[derive(Parser, Debug)]
pub struct Args {
    participants: PathBuf,
    #[arg(long)]
    history: Option<PathBuf>,
    #[arg(long, default_value_t = 10.0)]
    history_halflife: f64,
    #[arg(long)]
    verbosity: Option<usize>,
}

fn non_blanks(path: &Path) -> Result<Vec<String>> {
    let content = fs::read_to_string(path).with_context(|| format!("reading {path:?}"))?;
    Ok(content
        .lines()
        .filter(|i| !i.is_empty())
        .map(|s| s.to_string())
        .collect())
}

fn main() -> Result<()> {
    simple_logger::init_with_env().context("initializing logger")?;
    let args = Args::parse();

    let participants = non_blanks(&args.participants)?;
    let history = if let Some(hist_path) = &args.history {
        non_blanks(hist_path)?
    } else {
        vec![]
    };
    if participants.is_empty() {
        Err(anyhow!("participant list is empty"))
    } else {
        let selection = choose(&participants, &history, &args).context("choosing participant")?;
        info!("selection:{selection}");
        Ok(())
    }
}
