use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn calculate_day() -> Result<(), Box<dyn std::error::Error>> {
    macro_rules! run {
        ($day:literal, $expected_out:literal) => {
            let mut cmd = Command::cargo_bin("hrs")?;
            cmd.arg("./tests/data/hours.txt").arg($day);
            cmd.assert()
                .success()
                .stdout($expected_out);
        }
    }

    run!("1.3", "\
----
02:00 8-10 [TAG-1] desc
00:30 10-10.30 tagless desc
01:30 10.30-12 [TAG-1] desc, and some more desc
00:30 12.30-13 -\"-
01:00 13-14 another tagless desc
00:30 14-14.30 [TAG-1] desc
00:30 14.30-15 yet another tagless desc
02:15 16-18.15 [TAG-1] desc
----
06:45 [TAG-1]
01:00 another tagless desc
00:30 tagless desc
00:30 yet another tagless desc
----
08:45
");

    run!("2.3", "\
----
02:00 8-10 [TAG-1] desc
00:45 10-10.45 tagless desc
00:15 10.45-11 [TAG-2] [SECONDARY TAG] desc
02:00 11.15-13.15 [TAG-3] desc
02:00 13.30-15.30 -\"-
----
02:00 [TAG-1]
00:15 [TAG-2]
04:00 [TAG-3]
00:45 tagless desc
----
07:00
");

    run!("3.3", "\
----
01:15 9-10.15 [TAG-1] desc
01:15 10.15-11.30 tagless desc 1
00:45 11.45-12.30 [TAG-2] desc
00:15 12.30-12.45 tagless desc 2
01:30 12.45-14.15 [TAG-2] desc
01:15 15.15-16.30 -\"-
----
01:15 [TAG-1]
03:30 [TAG-2]
01:15 tagless desc 1
00:15 tagless desc 2
----
06:15
");

    run!("4.3", "\
----
----
----
00:00
");

    Ok(())
}

#[test]
fn file_doesnt_exist() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("hrs")?;

    cmd.arg("test/file/doesnt/exist").arg("1.3");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("could not read file"));

    Ok(())
}
