//! Print a warning if the Btrfs filesystem data usage drops below a free limit percentage.
//!
//! Usage:
//!
//!     ## btrfs-usage-monitor /mnt/btrfs 10
//!     WARNING /mnt/btrfs free: 752.58GiB (min: 681.47GiB), 9% (limit: 10%)

use anyhow::Result;
use clap::{crate_version, Parser};
use std::process;

mod btrfs;
mod config;

use btrfs::btrfs_usage;
use config::Config;

#[derive(Parser)]
#[clap(version = crate_version!())]
struct Opts {
    /// Prints debug information.
    #[clap(short, long)]
    debug: bool,

    /// Path to a subvolume or folder on the Btrfs filesystem.
    path: String,

    /// Maximum free filesystem data usage in percentage.
    free_limit_percentage: u8,
}

fn main() {
    // NOTE Parse errors (at least, "invalid value" errors) printed by Clap's parse() are missing a newline
    // let opts = Opts::parse();
    let opts = match Opts::try_parse() {
        Ok(opts) => opts,
        Err(err) => {
            eprintln!("{}", err);
            process::exit(2);
        }
    };

    let config = Config { debug: opts.debug };

    if let Err(err) = run(&config, &opts.path, opts.free_limit_percentage) {
        eprint!("error: {}", err);
        if let Some(err) = err.source() {
            eprint!(": {}", err);
        }
        eprintln!();
        process::exit(1);
    }
}

fn run(config: &Config, path: &str, free_limit_percentage: u8) -> Result<()> {
    if let Some(warning) = btrfs_usage(config, path, free_limit_percentage)? {
        println!("{}", warning);
    }
    Ok(())
}
