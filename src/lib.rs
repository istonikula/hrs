#![allow(unused)]

use chrono::{Duration, NaiveTime};
use lazy_static::lazy_static;
use regex::Regex;

use std::collections::HashMap;

pub fn find_lines_in_day<'a>(content: &'a str, date: &str) -> Vec<&'a str> {
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
) -> (Vec<(Duration, String)>, HashMap<String, Vec<Duration>>) {
    let mut durations: Vec<(Duration, String)> = Vec::new();
    let mut durations_by_tag: HashMap<String, Vec<Duration>> = HashMap::new();
    let mut prev_tag: Option<String> = None;
    for line in lines {
        process_line(line, &mut prev_tag, &mut durations, &mut durations_by_tag);
    }
    (durations, durations_by_tag)
}

fn process_line(
    line: &str,
    prev_tag: &mut Option<String>,
    durations: &mut Vec<(Duration, String)>,
    durations_by_tag: &mut HashMap<String, Vec<Duration>>,
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
                time.to_string()
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
        durations.push((duration, line.to_string()));
        durations_by_tag
            .entry(tag)
            .or_insert(Vec::new())
            .push(duration);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_lined_in_day() {
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

        assert_eq!(find_lines_in_day(&input, "27.2"), vec!["27.2", "--", "foo"]);
        assert_eq!(
            find_lines_in_day(&input, "28.2"),
            vec!["28.2 (day info)", "--", "bar"]
        );
        assert_eq!(find_lines_in_day(&input, "28.02"), [] as [&str; 0]);
        assert_eq!(
            find_lines_in_day(&input, "1.3  day info"),
            vec!["1.3  day info", "--", "baz"]
        );
    }

    #[test]
    fn test_process_line() {
        let mut prev_tag = None;
        let mut durations = Vec::new();
        let mut exp_durations = Vec::new();
        let mut by_tag = HashMap::new();

        macro_rules! line {
            ($line:tt) => {
                process_line($line, &mut prev_tag, &mut durations, &mut by_tag);
            };
        }

        let line = "8-9 desc without tag 1";
        let tag = &line[4..];
        line!(line);
        exp_durations.push((Duration::hours(1), line.to_string()));
        assert_eq!(prev_tag, Some(tag.to_string()));
        assert_eq!(durations, exp_durations);
        assert_eq!(by_tag[tag], vec![Duration::hours(1)]);

        let line = "9-9.30 [tag1] desc";
        let tag = "[tag1]";
        line!(line);
        exp_durations.push((Duration::minutes(30), line.to_string()));
        assert_eq!(prev_tag, Some(tag.to_string()));
        assert_eq!(durations, exp_durations);
        assert_eq!(by_tag[tag], vec![Duration::minutes(30)]);

        let line = "9.45-10 -\"-";
        line!(line);
        exp_durations.push((Duration::minutes(15), line.to_string()));
        assert_eq!(prev_tag, Some(tag.to_string()));
        assert_eq!(durations, exp_durations);
        assert_eq!(
            by_tag[tag],
            vec![Duration::minutes(30), Duration::minutes(15)]
        );

        let line = "10-10.30 desc without tag 2";
        let tag = &line[9..];
        line!(line);
        exp_durations.push((Duration::minutes(30), line.to_string()));
        assert_eq!(prev_tag, Some(tag.to_string()));
        assert_eq!(durations, exp_durations);
        assert_eq!(by_tag[tag], vec![Duration::minutes(30)]);

        let line = "10.45-11 -\"-";
        line!(line);
        exp_durations.push((Duration::minutes(15), line.to_string()));
        assert_eq!(prev_tag, Some(tag.to_string()));
        assert_eq!(durations, exp_durations);
        assert_eq!(
            by_tag[tag],
            vec![Duration::minutes(30), Duration::minutes(15)]
        );

        let line = "12-14.15 desc without tag 1";
        let tag = &line[9..];
        line!(line);
        exp_durations.push((Duration::minutes(135), line.to_string()));
        assert_eq!(prev_tag, Some(tag.to_string()));
        assert_eq!(durations, exp_durations);
        assert_eq!(
            by_tag[tag],
            vec![Duration::hours(1), Duration::minutes(135)]
        );

        let line = "14.15-16 [tag1] desc, with some additinal info";
        let tag = "[tag1]";
        line!(line);
        exp_durations.push((Duration::minutes(105), line.to_string()));
        assert_eq!(prev_tag, Some(tag.to_string()));
        assert_eq!(durations, exp_durations);
        assert_eq!(
            by_tag[tag],
            vec![
                Duration::minutes(30),
                Duration::minutes(15),
                Duration::minutes(105)
            ]
        );

        let line = "16-17   [tag1] NOTE: whitespace before tag";
        line!(line);
        exp_durations.push((Duration::minutes(60), line.to_string()));
        assert_eq!(prev_tag, Some(tag.to_string()));
        assert_eq!(durations, exp_durations);
        assert_eq!(
            by_tag[tag],
            vec![
                Duration::minutes(30),
                Duration::minutes(15),
                Duration::minutes(105),
                Duration::minutes(60),
            ]
        );
    }
}
