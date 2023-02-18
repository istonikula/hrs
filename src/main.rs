#![allow(unused)]

use anyhow::{Context, Result};
use chrono::Duration;
use clap::Parser;
use colored::Colorize;

use std::collections::HashMap;
use std::io::Write;

use grrs::{find_and_collect_day, write_durations_collect_total, human_duration};

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
        grrs::process_line(line, &mut prev_tag, &mut durations_by_tag, &out);
    }
    writeln!(out, "----");
    let duration_total = write_durations_collect_total(&durations_by_tag, &out);
    writeln!(out, "----");
    writeln!(out, "{}", human_duration(duration_total).bold().white());

    Ok(())
}

