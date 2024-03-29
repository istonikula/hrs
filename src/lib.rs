#![allow(unused)]

use chrono::{Duration, NaiveTime};
use colored::{ColoredString, Colorize};
use lazy_static::lazy_static;
use regex::Regex;

use std::collections::HashMap;
use std::fmt;

pub fn find_and_collect_day<'a>(content: &'a str, date: &str) -> Vec<&'a str> {
    let mut in_day = false;
    let mut lines_in_day = Vec::new();

    for line in content.lines() {
        if !in_day {
            if line == date || line.starts_with(&format!("{} ", date)) {
                in_day = true;
                lines_in_day.push(line);
            }
        } else {
            if line.is_empty() {
                break;
            }
            lines_in_day.push(line);
        }
    }
    lines_in_day
}

pub fn process_lines(
    lines: Vec<&str>,
    durations_by_tag: &mut HashMap<String, Vec<Duration>>,
    mut writer: impl std::io::Write,
) {
    let mut prev_tag: Option<String> = None;
    for line in lines {
        process_line(line, &mut prev_tag, durations_by_tag, &mut writer);
    }
}

fn process_line(
    line: &str,
    prev_tag: &mut Option<String>,
    durations_by_tag: &mut HashMap<String, Vec<Duration>>,
    mut writer: impl std::io::Write,
) {
    lazy_static! {
        static ref LINE_RE: Regex =
            Regex::new(r"^([0-9\.]{1,5})-([0-9\.]{1,5})\s+(\[.*?\])?.*$").unwrap();
    }

    let caps = LINE_RE.captures(line);

    if let Some(caps) = caps {
        let line = &caps[0];
        let start = &caps[1];
        let end = &caps[2];
        let mut tag = match caps.get(3) {
            Some(tag) => tag.as_str(),
            None => &line[format!("{}-{} ", start, end).len()..],
        }
        .to_owned();
        if let Some(ref pt) = prev_tag {
            if (tag.starts_with("-\"-")) {
                tag = pt.clone();
            } else {
                *prev_tag = Some(tag.clone());
            }
        } else {
            *prev_tag = Some(tag.clone());
        }

        fn with_mins(time: &str) -> String {
            if !time.contains('.') {
                format!("{}.00", time)
            } else {
                time.to_owned()
            }
        }
        let start = with_mins(start);
        let end = with_mins(end);

        fn parse(time: &str) -> NaiveTime {
            NaiveTime::parse_from_str(time, "%H.%M").unwrap()
        }
        let start = parse(&start);
        let end = parse(&end);
        let duration = end - start;
        let durations = durations_by_tag.entry(tag).or_insert(Vec::new());
        durations.push(duration);

        writeln!(writer, "{} {}", HumanDuration(duration).line(), &line);
    }
}

pub fn write_durations_collect_total(
    durations_by_tag: &HashMap<String, Vec<Duration>>,
    mut writer: impl std::io::Write,
) -> Duration {
    let mut durations_by_tag: Vec<_> = durations_by_tag.iter().collect();
    durations_by_tag.sort_by(|a, b| a.0.cmp(b.0));

    let mut duration_total = Duration::minutes(0);
    for (tag, durations) in durations_by_tag {
        let mut duration = Duration::minutes(0);
        for d in durations {
            duration = duration + *d;
        }
        duration_total = duration_total + duration;
        writeln!(writer, "{} {}", HumanDuration(duration).tag(), tag);
    }
    duration_total
}

pub fn write_total(total: Duration, mut writer: impl std::io::Write) {
    let full_day = Duration::hours(7) + Duration::minutes(30);
    let diff = total - full_day;

    if diff == Duration::zero() {
        writeln!(writer, "{}", HumanDuration(total).total());
        return;
    }

    writeln!(
        writer,
        "{} {}",
        HumanDuration(total).total(),
        HumanDuration(diff).diff()
    );
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

#[cfg(test)]
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

    #[test]
    fn test_find_and_collect_day() {
        let input = "


27.2
--
foo


28.2 (day info)
--
bar

1.3  day info
--
baz
";

        assert_eq!(
            find_and_collect_day(&input, "27.2"),
            vec!["27.2", "--", "foo"]
        );
        assert_eq!(
            find_and_collect_day(&input, "28.2"),
            vec!["28.2 (day info)", "--", "bar"]
        );
        assert_eq!(find_and_collect_day(&input, "28.02"), [] as [&str; 0]);
        assert_eq!(
            find_and_collect_day(&input, "1.3  day info"),
            vec!["1.3  day info", "--", "baz"]
        );
    }

    #[test]
    fn test_process_line() {
        let mut out = Vec::new();
        let mut prev_tag = None;
        let mut durations_by_tag = HashMap::new();

        macro_rules! line {
            ($line:tt) => {
                process_line($line, &mut prev_tag, &mut durations_by_tag, &mut out);
            };
        }
        let expected_output = |d: Duration, line: &str| {
            format!("{} {}\n", HumanDuration(d).line(), line).into_bytes()
        };

        let line = "8-9 desc without tag 1";
        line!(line);
        assert_eq!(out, expected_output(Duration::hours(1), line));
        assert_eq!(prev_tag, Some(line[4..].to_owned()));
        assert_eq!(durations_by_tag[&line[4..]], vec![Duration::hours(1)]);

        out.clear();

        let line = "9-9.30 [tag1] desc";
        line!(line);
        assert_eq!(out, expected_output(Duration::minutes(30), line));
        assert_eq!(prev_tag, Some("[tag1]".to_owned()));
        assert_eq!(durations_by_tag["[tag1]"], vec![Duration::minutes(30)]);

        out.clear();

        let line = "9.45-10 -\"-";
        line!(line);
        assert_eq!(out, expected_output(Duration::minutes(15), line));
        assert_eq!(prev_tag, Some("[tag1]".to_owned()));
        assert_eq!(
            durations_by_tag["[tag1]"],
            vec![Duration::minutes(30), Duration::minutes(15)]
        );

        out.clear();

        let line = "10-10.30 desc without tag 2";
        line!(line);
        assert_eq!(out, expected_output(Duration::minutes(30), line));
        assert_eq!(prev_tag, Some(line[9..].to_owned()));
        assert_eq!(durations_by_tag[&line[9..]], vec![Duration::minutes(30)]);

        out.clear();

        line!("10.45-11 -\"-");
        assert_eq!(out, expected_output(Duration::minutes(15), "10.45-11 -\"-"));
        assert_eq!(prev_tag, Some(line[9..].to_owned()));
        assert_eq!(
            durations_by_tag[&line[9..]],
            vec![Duration::minutes(30), Duration::minutes(15)]
        );

        out.clear();

        let line = "12-14.15 desc without tag 1";
        line!(line);
        assert_eq!(out, expected_output(Duration::minutes(135), line));
        assert_eq!(prev_tag, Some(line[9..].to_owned()));
        assert_eq!(
            durations_by_tag[&line[9..]],
            vec![Duration::hours(1), Duration::minutes(135)]
        );

        out.clear();

        let line = "14.15-16 [tag1] desc, with some additinal info";
        line!(line);
        assert_eq!(out, expected_output(Duration::minutes(105), line));
        assert_eq!(prev_tag, Some("[tag1]".to_owned()));
        assert_eq!(
            durations_by_tag["[tag1]"],
            vec![
                Duration::minutes(30),
                Duration::minutes(15),
                Duration::minutes(105)
            ]
        );

        out.clear();

        let line = "16-17   [tag1] NOTE: whitespace before tag";
        line!(line);
        assert_eq!(out, expected_output(Duration::minutes(60), line));
        assert_eq!(prev_tag, Some("[tag1]".to_owned()));
        assert_eq!(
            durations_by_tag["[tag1]"],
            vec![
                Duration::minutes(30),
                Duration::minutes(15),
                Duration::minutes(105),
                Duration::minutes(60),
            ]
        );
    }

    #[test]
    fn test_write_duration_collect_total() {
        let mut out = Vec::new();
        let durations_by_tag = HashMap::from([
            (
                "desc without tag 2".to_owned(),
                vec![Duration::minutes(30), Duration::minutes(15)],
            ),
            (
                "[tag1]".to_owned(),
                vec![
                    Duration::minutes(30),
                    Duration::minutes(15),
                    Duration::minutes(105),
                ],
            ),
            (
                "desc without tag 1".to_owned(),
                vec![Duration::hours(1), Duration::minutes(135)],
            ),
            (
                "[tag2]".to_owned(),
                vec![Duration::hours(1), Duration::minutes(45)],
            ),
        ]);

        fn sum(durations: &Vec<Duration>) -> Duration {
            durations.iter().fold(Duration::zero(), |acc, &x| acc + x)
        }
        macro_rules! sum_by_tag {
            ($tag:literal) => {
                HumanDuration(sum(&durations_by_tag[$tag])).tag()
            };
        }

        let total = write_durations_collect_total(&durations_by_tag, &mut out);
        let expected_out = vec![
            format!("{} [tag1]\n", sum_by_tag!("[tag1]")),
            format!("{} [tag2]\n", sum_by_tag!("[tag2]")),
            format!("{} desc without tag 1\n", sum_by_tag!("desc without tag 1")),
            format!("{} desc without tag 2\n", sum_by_tag!("desc without tag 2")),
        ]
        .join("");
        assert_eq!(out, expected_out.as_bytes());
        assert_eq!(
            total,
            sum(&durations_by_tag.into_values().flatten().collect())
        );
    }

    #[test]
    fn test_write_total() {
        let mut out = Vec::new();

        let total = HumanDuration(Duration::hours(7));
        let diff = HumanDuration(Duration::minutes(-30));
        write_total(total.0, &mut out);
        assert_eq!(
            out,
            format!("{} {}\n", total.total(), diff.diff()).as_bytes()
        );

        out.clear();

        let total = HumanDuration(Duration::hours(7) + Duration::minutes(30));
        let diff = HumanDuration(Duration::zero());
        write_total(total.0, &mut out);
        assert_eq!(out, format!("{}\n", total.total()).as_bytes());

        out.clear();

        let total = HumanDuration(Duration::hours(8));
        let diff = HumanDuration(Duration::minutes(30));
        write_total(total.0, &mut out);
        assert_eq!(
            out,
            format!("{} {}\n", total.total(), diff.diff()).as_bytes()
        );
    }
}
