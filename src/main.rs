#![allow(unused)]

use anyhow::{Context, Result};
use chrono::Duration;
use clap::Parser;
use colored::Colorize;

use std::collections::HashMap;
use std::io::Write;

use hrs::{find_and_collect_day, human_duration, human_duration_signed, process_line, write_durations_collect_total};

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

    let mut durations_by_tag: HashMap<String, Vec<Duration>> = HashMap::new();
    let mut prev_tag: Option<String> = None;
    let mut out = std::io::stdout();

    writeln!(out, "----");
    for line in lines_in_day {
        process_line(line, &mut prev_tag, &mut durations_by_tag, &out);
    }
    writeln!(out, "----");
    let duration_total = write_durations_collect_total(&durations_by_tag, &out);
    writeln!(out, "----");
    // TODO move to function
    let full_day = Duration::hours(7) + Duration::minutes(30);
    let diff = duration_total - full_day;
    let diff = if diff < Duration::zero() { human_duration_signed(diff).red() } else { human_duration_signed(diff).green() }; 

    writeln!(out, "{} {}", human_duration(duration_total).bold().white(), diff);

    Ok(())
}

