#![allow(unused)]

use anyhow::{Context, Result};
use chrono::Duration;
use clap::Parser;
use colored::Colorize;

use std::collections::HashMap;
use std::io::{self, Write};

use hrs::{find_and_collect_day, process_lines, summarize_durations, output::{HumanDuration, print_processed_lines, print_summary, print_total_and_diff}};

#[derive(Parser)]
struct Cli {
    path: std::path::PathBuf,
    date: String,
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let content = std::fs::read_to_string(&args.path)
        .with_context(|| format!("could not read file `{}`", args.path.display()))?;

    let lines_in_day = find_and_collect_day(&content, &args.date);
    let (processed_lines, durations_by_tag) = process_lines(lines_in_day);
    let (summary, duration_total) = summarize_durations(&durations_by_tag);

    let mut out = std::io::stdout();
    print_processed_lines(&mut out, processed_lines)?;
    print_summary(&mut out, summary)?;
    print_total_and_diff(&mut out, duration_total)?;

    Ok(())
}
