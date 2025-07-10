use anyhow::Result;
use chrono::Duration;
use colored::{ColoredString, Colorize};
use std::fmt;
use std::io::Write;

pub struct HumanDuration(pub Duration);
impl HumanDuration {
    pub fn plain(&self) -> String {
        format!("{}", self)
    }
    pub fn line(&self) -> ColoredString {
        self.plain().bold().green()
    }
    pub fn tag(&self) -> ColoredString {
        self.plain().bold().blue()
    }
    pub fn total(&self) -> ColoredString {
        self.plain().bold().white()
    }
    pub fn diff(&self) -> ColoredString {
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

pub fn print_total_and_diff(
    mut writer: impl std::io::Write,
    total: chrono::Duration,
) -> Result<()> {
    writeln!(writer, "----");
    let full_day = Duration::hours(7) + Duration::minutes(30);
    let diff = total - full_day;

    if diff == Duration::zero() {
        writeln!(writer, "{}", HumanDuration(total).total())?;
    } else {
        writeln!(
            writer,
            "{} {}",
            HumanDuration(total).total(),
            HumanDuration(diff).diff()
        )?;
    }
    Ok(())
}

pub fn print_summary(
    mut writer: impl std::io::Write,
    summary: Vec<(String, chrono::Duration)>,
) -> Result<()> {
    writeln!(writer, "----");
    for (tag, duration) in summary {
        writeln!(writer, "{} {}", HumanDuration(duration).tag(), tag)?;
    }
    Ok(())
}

pub fn print_processed_lines(
    mut writer: impl std::io::Write,
    processed_lines: Vec<(chrono::Duration, &str)>,
) -> Result<()> {
    writeln!(writer, "----");
    for (duration, line) in processed_lines {
        writeln!(writer, "{} {}", HumanDuration(duration).line(), &line)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_human_duration() -> Result<()> {
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
        Ok(())
    }

    #[test]
    fn test_print_processed_lines() -> Result<()> {
        let mut out = Vec::new();
        let processed_lines = vec![
            (Duration::hours(1), "8-9 [TAG] desc"),
            (Duration::minutes(30), "9-9.30 tagless"),
        ];

        print_processed_lines(&mut out, processed_lines.clone())?;

        assert_eq!(
            out,
            format!(
                "----\n{} {}\n{} {}\n",
                HumanDuration(processed_lines[0].0).line(),
                processed_lines[0].1,
                HumanDuration(processed_lines[1].0).line(),
                processed_lines[1].1
            )
            .into_bytes(),
        );
        Ok(())
    }

    #[test]
    fn test_print_summary() -> Result<()> {
        let mut out = Vec::new();
        let summary = vec![
            ("TAG1".to_owned(), Duration::hours(2)),
            ("TAG2".to_owned(), Duration::minutes(45)),
        ];

        print_summary(&mut out, summary.clone())?;

        assert_eq!(
            out,
            format!(
                "----\n{} {}\n{} {}\n",
                HumanDuration(summary[0].1).tag(),
                summary[0].0,
                HumanDuration(summary[1].1).tag(),
                summary[1].0
            )
            .into_bytes()
        );
        Ok(())
    }

    #[test]
    fn test_print_total_and_diff() -> Result<()> {
        let mut out = Vec::new();
        let total = Duration::hours(8);

        print_total_and_diff(&mut out, total)?;

        assert_eq!(
            out,
            format!(
                "----\n{} {}\n",
                HumanDuration(total).total(),
                HumanDuration(Duration::minutes(30)).diff()
            )
            .into_bytes()
        );
        Ok(())
    }

    #[test]
    fn test_print_total_and_diff_zero() -> Result<()> {
        let mut out = Vec::new();
        let total = Duration::hours(7) + Duration::minutes(30);

        print_total_and_diff(&mut out, total)?;

        assert_eq!(
            out,
            format!("----\n{}\n", HumanDuration(total).total()).into_bytes()
        );
        Ok(())
    }

    #[test]
    fn test_print_total_and_diff_negative() -> Result<()> {
        let mut out = Vec::new();
        let total = Duration::hours(6);

        print_total_and_diff(&mut out, total)?;

        assert_eq!(
            out,
            format!(
                "----\n{} {}\n",
                HumanDuration(total).total(),
                HumanDuration(Duration::hours(-1) + Duration::minutes(-30)).diff()
            )
            .into_bytes()
        );
        Ok(())
    }
}
