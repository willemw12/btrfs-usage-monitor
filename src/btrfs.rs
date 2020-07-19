use anyhow::{bail, ensure, Context, Result};
use regex::Regex;
use std::process::{Command, Stdio};

/// Human readable data usage (e.g., 1K 234M 2G).
#[derive(Clone, Debug)]
struct UsageHuman {
    free: String,
    free_min: String,
}

/// Raw data usage in bytes.
#[derive(Debug)]
struct UsageRaw {
    device_size: u64,
    free: u64,
}

/// Returns a warning if Btrfs data usage drops below the free limit percentage.
pub fn btrfs_usage(path: &str, free_limit_percentage: u64) -> Result<Option<String>> {
    let usage_raw = usage_raw(path)?;
    let usage_human = usage_human(path)?;
    usage_warning(path, free_limit_percentage, &usage_raw, &usage_human)
}

//

fn usage_raw(path: &str) -> Result<UsageRaw> {
    let output = Command::new("btrfs")
        .arg("filesystem")
        .arg("usage")
        .arg("--raw")
        .arg(path)
        .stderr(Stdio::inherit())
        .output()
        .context("failed to execute btrfs")?;
    // if !output.status.success() {
    //     // return Err(anyhow!("btrfs: {}", output.status));
    //     bail!("btrfs: {}", output.status);
    // }
    ensure!(output.status.success(), "btrfs: {}", output.status);

    let buf = String::from_utf8(output.stdout).context("usage data parse error")?;
    extract_usage_raw(&buf)
}

fn usage_human(path: &str) -> Result<UsageHuman> {
    let output = Command::new("btrfs")
        .arg("filesystem")
        .arg("usage")
        .arg(path)
        .stderr(Stdio::inherit())
        .output()
        .context("failed to execute btrfs")?;
    ensure!(output.status.success(), "btrfs: {}", output.status);

    let buf = String::from_utf8(output.stdout).context("usage data parse error")?;
    extract_usage_human(&buf)
}

fn usage_warning(
    path: &str,
    free_limit_percentage: u64,
    usage_raw: &UsageRaw,
    usage_human: &UsageHuman,
) -> Result<Option<String>> {
    let free_percentage = (usage_raw.free * 100) / usage_raw.device_size;

    Ok(if free_percentage < free_limit_percentage {
        Some(
            // format!("WARNING {}: {} (min: {})", path, usage_human.free, usage_human.free_min),
            format!(
                "WARNING {} free: {} (min: {}), {}% (limit: {}%)",
                path,
                usage_human.free,
                usage_human.free_min,
                free_percentage,
                free_limit_percentage
            ),
        )
    } else {
        None
    })
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
        Some(u) => *u,
        None => bail!("usage data parse error: Device size in raw bytes"),
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
        None => bail!("usage data parse error: Free (estimated) in raw bytes"),
    };

    if device_size == 0 {
        bail!("usage data parse error");
    }

    Ok(UsageRaw {
        device_size: device_size,
        free: free,
    })
}

fn extract_usage_human(buf: &str) -> Result<UsageHuman> {
    // let pattern = Regex::new(r"Free \(estimated\):\s*(.*)\b\s*\(min:\s*(.*)\s*\)")?;
    let pattern = Regex::new(r"Free \(estimated\):(.*)\(min:(.*)\)")?;
    let result = buf
        .lines()
        .filter_map(|line| pattern.captures(line))
        .filter(|cap| cap.len() > 2)
        .map(|cap| UsageHuman {
            free: cap[1].trim().to_string(),
            free_min: cap[2].trim().to_string(),
        })
        .collect::<Vec<_>>();

    match result.first() {
        Some(u) => Ok(u.clone()),
        None => bail!("usage data parse error: Free (estimated)"),
    }
}

//

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn test_btrfs_usage1() {
        // Get input
        const PATH: &str = "/mnt/btrfs";
        const FREE_LIMIT_PERCENTAGE: u64 = 60;
        let buf_raw = fs::read_to_string("tests/input-usage-raw1.txt").expect("ERROR");
        let buf_human = fs::read_to_string("tests/input-usage-human1.txt").expect("ERROR");

        // Run test
        let usage_raw = extract_usage_raw(&buf_raw);
        let usage_human = extract_usage_human(&buf_human);
        let warning = usage_warning(
            PATH,
            FREE_LIMIT_PERCENTAGE,
            &usage_raw.unwrap(),
            &usage_human.unwrap(),
        );

        // Check result
        let expected = fs::read_to_string("tests/output-warning1.txt").expect("ERROR");
        // NOTE fs::read_to_string() adds a newline character at the end of the string
        let expected = expected.trim_end();
        assert_eq!(warning.unwrap().unwrap(), expected);
    }
}
