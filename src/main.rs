#![allow(unused)]

use anyhow::{Context, Result};
use chrono::Duration;
use clap::Parser;
use colored::Colorize;

use std::collections::HashMap;
use std::io::Write;

use hrs::{find_and_collect_day, process_lines, summarize_durations, HumanDuration};

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

fn print_total_and_diff(
    out: &mut std::io::Stdout,
    duration_total: chrono::Duration,
) -> Result<()> {
    writeln!(out, "----");
    let full_day = Duration::hours(7) + Duration::minutes(30);
    let diff = duration_total - full_day;

    if diff == Duration::zero() {
        writeln!(out, "{}", HumanDuration(duration_total).total())?;
    } else {
        writeln!(
            out,
            "{} {}",
            HumanDuration(duration_total).total(),
            HumanDuration(diff).diff()
        )?;
    }
    Ok(())
}

fn print_summary(
    out: &mut std::io::Stdout,
    summary: Vec<(String, chrono::Duration)>,
) -> Result<()> {
    writeln!(out, "----");
    for (tag, duration) in summary {
        writeln!(out, "{} {}", HumanDuration(duration).tag(), tag)?;
    }
    Ok(())
}

fn print_processed_lines(
    out: &mut std::io::Stdout,
    processed_lines: Vec<(chrono::Duration, &str)>,
) -> Result<()> {
    writeln!(out, "----");
    for (duration, line) in processed_lines {
        writeln!(out, "{} {}", HumanDuration(duration).line(), &line)?;
    }
    Ok(())
}
