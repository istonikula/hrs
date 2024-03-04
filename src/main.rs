#![allow(unused)]

use anyhow::{Context, Result};
use chrono::Duration;
use clap::Parser;
use colored::{ColoredString, Colorize};

use std::collections::HashMap;
use std::fmt;
use std::io::Write;

use hrs::{find_lines_in_day, process_lines};

#[derive(Parser)]
struct Cli {
    path: std::path::PathBuf,
    date: String,
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let content = std::fs::read_to_string(&args.path)
        .with_context(|| format!("could not read file `{}`", args.path.display()))?;

    let mut out = std::io::stdout();

    let lines_in_day = find_lines_in_day(&content, &args.date);
    let (durations, by_tag) = process_lines(lines_in_day);

    let mut by_tag: Vec<_> = by_tag.iter().collect();
    by_tag.sort_by(|a, b| a.0.cmp(b.0));

    let total = durations.iter().fold(Duration::zero(), |acc, x| acc + x.0);
    let full_day = Duration::hours(7) + Duration::minutes(30);
    let diff = total - full_day;

    writeln!(out, "----");
    for (duration, line) in durations {
        writeln!(out, "{} {}", HumanDuration(duration).line(), &line);
    }

    writeln!(out, "----");
    for (tag, durations) in by_tag {
        writeln!(
            out,
            "{} {}",
            HumanDuration(durations.iter().sum()).tag(),
            tag
        );
    }

    writeln!(out, "----");
    if diff == Duration::zero() {
        writeln!(out, "{}", HumanDuration(total).total());
    } else {
        writeln!(
            out,
            "{} {}",
            HumanDuration(total).total(),
            HumanDuration(diff).diff()
        );
    }

    Ok(())
}

struct HumanDuration(Duration);
impl HumanDuration {
    fn plain(&self) -> String {
        format!("{}", self)
    }
    fn line(&self) -> ColoredString {
        self.plain().bold().green()
    }
    fn tag(&self) -> ColoredString {
        self.plain().bold().blue()
    }
    fn total(&self) -> ColoredString {
        self.plain().bold().white()
    }
    fn diff(&self) -> ColoredString {
        if self.0 < Duration::zero() {
            format!("-{}", self).red()
        } else {
            format!("+{}", self).green()
        }
    }
}
impl fmt::Display for HumanDuration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:02}:{:02}",
            self.0.num_hours().abs(),
            self.0.num_minutes().abs() % 60
        )
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_human_duration() {
        assert_eq!(HumanDuration(Duration::minutes(1)).plain(), "00:01");
        assert_eq!(HumanDuration(Duration::minutes(15)).plain(), "00:15");
        assert_eq!(HumanDuration(Duration::hours(1)).plain(), "01:00");
        assert_eq!(HumanDuration(Duration::minutes(135)).plain(), "02:15");
        assert_eq!(HumanDuration(Duration::hours(10)).plain(), "10:00");

        let hour = (Duration::hours(1), "01:00");
        assert_eq!(HumanDuration(hour.0).line(), hour.1.bold().green());
        assert_eq!(HumanDuration(hour.0).tag(), hour.1.bold().blue());
        assert_eq!(HumanDuration(hour.0).total(), hour.1.bold().white());
        assert_eq!(HumanDuration(hour.0).diff(), format!("+{}", hour.1).green());
        assert_eq!(HumanDuration(-hour.0).diff(), format!("-{}", hour.1).red());
    }
}
