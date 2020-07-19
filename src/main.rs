//! Print a warning if Btrfs data usage drops below the free limit percentage.
//!
//! Usage:
//!
//!     ## btrfs-usage-monitor /mnt/btrfs 10
//!     WARNING /mnt/btrfs free: 752.58GiB (min: 681.47GiB), 9% (limit: 10%)

use anyhow::Result;
use clap::Clap;
use std::process;

mod btrfs;

use btrfs::btrfs_usage;

#[derive(Clap)]
#[clap(name = "")]
struct Opts {
    path: String,
    free_limit_percentage: u64,
}

fn main() {
    // NOTE Parse errors (at least, "invalid value" errors) printed by Clap's parse() are missing a newline
    // let opts: Opts = Opts::parse();
    let opts: Opts = match Opts::try_parse() {
        Ok(opts) => opts,
        Err(err) => {
            eprintln!("{}", err);
            process::exit(2);
        }
    };
    if let Err(err) = run(&opts.path, opts.free_limit_percentage) {
        eprint!("error: {}", err);
        if let Some(err) = err.source() {
            eprint!(": {}", err);
        }
        eprintln!();
        process::exit(1);
    }
}

fn run(path: &str, free_limit_percentage: u64) -> Result<()> {
    if let Some(warning) = btrfs_usage(path, free_limit_percentage)? {
        println!("{}", warning);
    }
    Ok(())
}
