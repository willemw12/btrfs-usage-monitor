use anyhow::{bail, ensure, Context, Result};
use regex::Regex;
use std::cmp;
use std::process::{Command, Stdio};

use super::config::Config;

/// Raw Btrfs filesystem data usage in bytes.
#[derive(Debug)]
struct UsageRaw {
    device_size: u64,
    free: u64,
}

/// Human readable Btrfs filesystem data usage (e.g., 1K 234M 2G).
#[derive(Clone, Debug)]
struct UsageHuman {
    free: String,
    free_min: String,
}

/// Returns a warning if Btrfs filesystem data usage drops below the free limit percentage.
/// Path is the Btrfs filesystem location.
pub fn btrfs_usage(
    config: &Config,
    path: &str,
    free_limit_percentage: u8,
) -> Result<Option<String>> {
    Ok(usage_warning(
        path,
        free_limit_percentage,
        &usage_raw(config, path)?,
        &usage_human(config, path)?,
    ))
}

//

fn usage_raw(config: &Config, path: &str) -> Result<UsageRaw> {
    let cmd = "btrfs";
    let args = vec!["filesystem", "usage", "--raw", path];
    let output = Command::new(cmd)
        .args(&args)
        .stderr(Stdio::inherit())
        .output()
        .context(format!("'{} {}'", cmd, args.join(" ")))?;
    // if !output.status.success() {
    //     // return Err(anyhow!("'{} {}': {}", cmd, args.join(" "),output.status));
    //     bail!("'{} {}': {}", cmd, args.join(" "),output.status);
    // }
    ensure!(
        output.status.success(),
        "'{} {}': {}",
        cmd,
        args.join(" "),
        output.status
    );

    // let buf = String::from_utf8(output.stdout).context("parse error in filesystem data usage")?;
    let buf = String::from_utf8_lossy(&output.stdout);
    if config.debug {
        eprint!("DEBUG: '{} {}':\n{}", cmd, args.join(" "), buf);
    }

    extract_usage_raw(&buf)
}

fn usage_human(config: &Config, path: &str) -> Result<UsageHuman> {
    let cmd = "btrfs";
    let args = vec!["filesystem", "usage", path];
    let output = Command::new(cmd)
        .args(&args)
        .stderr(Stdio::inherit())
        .output()
        .context(format!("'{} {}'", cmd, args.join(" ")))?;
    ensure!(
        output.status.success(),
        "'{} {}': {}",
        cmd,
        args.join(" "),
        output.status
    );

    let buf = String::from_utf8_lossy(&output.stdout);
    if config.debug {
        eprint!("DEBUG: '{} {}':\n{}", cmd, args.join(" "), buf);
    }

    extract_usage_human(&buf)
}

fn usage_warning(
    path: &str,
    free_limit_percentage: u8,
    usage_raw: &UsageRaw,
    usage_human: &UsageHuman,
) -> Option<String> {
    // // ensure!(usage_raw.device_size > 0, "btrfs: device size is 0");
    if usage_raw.device_size == 0 {
        return Some(format!("ERROR: {}: device size is 0", path));
    }

    let free_limit_percentage = free_limit_percentage.clamp(0, 100);

    let free_percentage = cmp::min((usage_raw.free * 100) / usage_raw.device_size, 100) as u8;

    if free_percentage < free_limit_percentage {
        Some(
            // format!("WARNING: {}, free: {} (min: {})", path, usage_human.free, usage_human.free_min),
            format!(
                "WARNING: {}, free: {} (min: {}), {}% (limit: {}%)",
                path,
                usage_human.free,
                usage_human.free_min,
                free_percentage,
                free_limit_percentage
            ),
        )
    } else {
        None
    }
}

//

fn extract_usage_raw(buf: &str) -> Result<UsageRaw> {
    let pattern = Regex::new(r"Device size:(.*)")?;
    let result = buf
        .lines()
        .filter_map(|line| pattern.captures(line))
        .filter(|cap| cap.len() > 1)
        .map(|cap| cap[1].trim().parse().unwrap_or_default())
        .collect::<Vec<u64>>();
    let device_size = match result.first() {
        // Some(0) => bail!("btrfs: device size is 0"),
        Some(u) => *u,
        None => bail!("parse error in filesystem data usage: at 'Device size' in raw bytes"),
    };

    let pattern = Regex::new(r"Free \(estimated\):(.*)\(")?;
    let result = buf
        .lines()
        .filter_map(|line| pattern.captures(line))
        .filter(|cap| cap.len() > 1)
        .map(|cap| cap[1].trim().parse().unwrap_or_default())
        .collect::<Vec<u64>>();
    let free = match result.first() {
        Some(u) => *u,
        None => bail!("parse error in filesystem data usage: at 'Free (estimated)' in raw bytes"),
    };

    Ok(UsageRaw { device_size, free })
}

fn extract_usage_human(buf: &str) -> Result<UsageHuman> {
    // let pattern = Regex::new(r"Free \(estimated\):\s*(.*)\b\s*\(min:\s*(.*)\s*\)")?;
    let pattern = Regex::new(r"Free \(estimated\):(.*)\(min:(.*)\)")?;
    let result = buf
        .lines()
        .filter_map(|line| pattern.captures(line))
        .filter(|cap| cap.len() > 2)
        .map(|cap| UsageHuman {
            free: String::from(cap[1].trim()),
            free_min: String::from(cap[2].trim()),
        })
        .collect::<Vec<_>>();

    match result.first() {
        Some(u) => Ok(u.to_owned()),
        None => bail!("parse error in filesystem data usage: at 'Free (estimated)'"),
    }
}

//

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    const PATH: &str = "/mnt/btrfs";

    #[derive(Default)]
    struct TestData {
        path: &'static str,
        input_file_raw: &'static str,
        input_file_human: &'static str,
        free_limit_percentage: u8,

        expected_file: &'static str,
    }

    macro_rules! test_set {
        ($name:ident, $data:expr) => {
            #[test]
            fn $name() {
                let data = $data;

                // Get input
                let buf_raw = fs::read_to_string(data.input_file_raw).expect("ERROR");
                let buf_human = fs::read_to_string(data.input_file_human).expect("ERROR");

                // Run test
                let usage_raw = extract_usage_raw(&buf_raw).expect("ERROR");
                let usage_human = extract_usage_human(&buf_human).expect("ERROR");
                let warning = usage_warning(
                    data.path,
                    data.free_limit_percentage,
                    &usage_raw,
                    &usage_human,
                );

                // Check result
                let expected = fs::read_to_string(data.expected_file).expect("ERROR");
                // NOTE fs::read_to_string() adds a newline character at the end of the string
                let expected = expected.trim_end();
                assert_eq!(warning.unwrap(), expected);
            }
        };
    }

    test_set!(
        test_btrfs_zero_size,
        TestData {
            path: PATH,
            input_file_raw: "tests/input-usage-raw-zero-size.txt",
            input_file_human: "tests/input-usage-human-zero-size.txt",
            expected_file: "tests/output-error-zero-size.txt",
            ..Default::default()
        }
    );

    test_set!(
        test_btrfs_usage1,
        TestData {
            path: PATH,
            input_file_raw: "tests/input-usage-raw1.txt",
            input_file_human: "tests/input-usage-human1.txt",
            free_limit_percentage: 60,
            expected_file: "tests/output-warning1.txt",
        }
    );

    //

    // macro_rules! test_set_fail {
    //     ($name:ident, $data:expr) => {
    //         #[test]
    //         #[should_panic]
    //         fn $name() {
    //             let data = $data;
    //
    //             // Get input
    //             let buf_raw = fs::read_to_string(data.input_file_raw).expect("ERROR");
    //
    //             // Run test
    //             extract_usage_raw(&buf_raw).unwrap();
    //         }
    //     };
    // }
    //
    // test_set_fail!(
    //     test_btrfs_zero_size,
    //     TestData {
    //         input_file_raw: "tests/input-usage-raw-zero-size.txt",
    //         ..Default::default()
    //     }
    // );
}
