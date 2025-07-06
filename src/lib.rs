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

pub fn process_lines<'a>(
    lines: Vec<&'a str>,
) -> (Vec<(Duration, &'a str)>, HashMap<String, Vec<Duration>>) {
    let mut durations_by_tag: HashMap<String, Vec<Duration>> = HashMap::new();
    let mut prev_tag: Option<String> = None;
    let mut processed_lines = Vec::new();
    for line in lines {
        if let Some(duration) = process_line(line, &mut prev_tag, &mut durations_by_tag) {
            processed_lines.push((duration, line));
        }
    }
    (processed_lines, durations_by_tag)
}

fn process_line(
    line: &str,
    prev_tag: &mut Option<String>,
    durations_by_tag: &mut HashMap<String, Vec<Duration>>,
) -> Option<Duration> {
    lazy_static! {
        static ref LINE_RE: Regex =
            Regex::new(r"^([0-9\.]{1,5})-([0-9\.]{1,5})\s+(\[.*?\])?.*$").unwrap();
    }

    let caps = LINE_RE.captures(line);

    if let Some(caps) = caps {
        let line_match = &caps[0];
        let start = &caps[1];
        let end = &caps[2];
        let mut tag = match caps.get(3) {
            Some(tag) => tag.as_str(),
            None => &line_match[format!("{}-{} ", start, end).len()..],
        }
        .to_owned();
        if let Some(ref pt) = prev_tag {
            if (tag.starts_with("-\"")) {
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

        Some(duration)
    } else {
        None
    }
}

pub fn summarize_durations(
    durations_by_tag: &HashMap<String, Vec<Duration>>,
) -> (Vec<(String, Duration)>, Duration) {
    let mut durations_by_tag: Vec<_> = durations_by_tag.iter().collect();
    durations_by_tag.sort_by(|a, b| a.0.cmp(b.0));

    let mut duration_total = Duration::minutes(0);
    let mut summary = Vec::new();
    for (tag, durations) in durations_by_tag {
        let mut duration = Duration::minutes(0);
        for d in durations {
            duration = duration + *d;
        }
        duration_total = duration_total + duration;
        summary.push((tag.clone(), duration));
    }
    (summary, duration_total)
}

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
        let mut prev_tag = None;
        let mut durations_by_tag = HashMap::new();

        macro_rules! line {
            ($line:tt) => {
                process_line($line, &mut prev_tag, &mut durations_by_tag).unwrap()
            };
        }

        let line = "8-9 desc without tag 1";
        let duration = line!(line);
        assert_eq!(duration, Duration::hours(1));
        assert_eq!(prev_tag, Some("desc without tag 1".to_owned()));
        assert_eq!(
            durations_by_tag["desc without tag 1"],
            vec![Duration::hours(1)]
        );

        let line = "9-9.30 [tag1] desc";
        let duration = line!(line);
        assert_eq!(duration, Duration::minutes(30));
        assert_eq!(prev_tag, Some("[tag1]".to_owned()));
        assert_eq!(
            durations_by_tag["[tag1]"],
            vec![Duration::minutes(30)]
        );

        let line = "9.45-10 -\"-";
        let duration = line!(line);
        assert_eq!(duration, Duration::minutes(15));
        assert_eq!(prev_tag, Some("[tag1]".to_owned()));
        assert_eq!(
            durations_by_tag["[tag1]"],
            vec![Duration::minutes(30), Duration::minutes(15)]
        );

        let line = "10-10.30 desc without tag 2";
        let duration = line!(line);
        assert_eq!(duration, Duration::minutes(30));
        assert_eq!(prev_tag, Some("desc without tag 2".to_owned()));
        assert_eq!(
            durations_by_tag["desc without tag 2"],
            vec![Duration::minutes(30)]
        );

        let duration = line!("10.45-11 -\"-");
        assert_eq!(duration, Duration::minutes(15));
        assert_eq!(prev_tag, Some("desc without tag 2".to_owned()));
        assert_eq!(
            durations_by_tag["desc without tag 2"],
            vec![Duration::minutes(30), Duration::minutes(15)]
        );

        let line = "12-14.15 desc without tag 1";
        let duration = line!(line);
        assert_eq!(duration, Duration::minutes(135));
        assert_eq!(prev_tag, Some("desc without tag 1".to_owned()));
        assert_eq!(
            durations_by_tag["desc without tag 1"],
            vec![Duration::hours(1), Duration::minutes(135)]
        );

        let line = "14.15-16 [tag1] desc, with some additinal info";
        let duration = line!(line);
        assert_eq!(duration, Duration::minutes(105));
        assert_eq!(prev_tag, Some("[tag1]".to_owned()));
        assert_eq!(
            durations_by_tag["[tag1]"],
            vec![
                Duration::minutes(30),
                Duration::minutes(15),
                Duration::minutes(105)
            ]
        );

        let line = "16-17   [tag1] NOTE: whitespace before tag";
        let duration = line!(line);
        assert_eq!(duration, Duration::minutes(60));
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
    fn test_summarize_durations() {
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

        let (summary, total) = summarize_durations(&durations_by_tag);

        let expected_summary = vec![
            (
                "[tag1]".to_owned(),
                sum(&durations_by_tag["[tag1]"]),
            ),
            (
                "[tag2]".to_owned(),
                sum(&durations_by_tag["[tag2]"]),
            ),
            (
                "desc without tag 1".to_owned(),
                sum(&durations_by_tag["desc without tag 1"]),
            ),
            (
                "desc without tag 2".to_owned(),
                sum(&durations_by_tag["desc without tag 2"]),
            ),
        ];

        assert_eq!(summary, expected_summary);
        assert_eq!(
            total,
            sum(&durations_by_tag.into_values().flatten().collect())
        );
    }
}
