Btrfs disk space usage monitor
==============================

A simple Btrfs disk space usage monitor.

There is also a version written in [Go](https://github.com/willemw12/btrfs-usage-monitor-go).


Feature
-------

- Print a warning if the Btrfs filesystem data usage drops below a free limit percentage.


Installation
------------

The following steps require that [Rust](https://www.rust-lang.org/) is installed. The install path used here ($HOME/bin) is an example.

To install from crates.io, run:

    $ cargo install btrfs-usage-monitor

To install to a specific folder, do one of the following.

Install from crates.io to $HOME/bin:

    $ cargo install --root $HOME btrfs-usage-monitor

Install from GitHub to $HOME/bin, for example, the latest commit:

    $ cargo install --root $HOME --git https://github.com/willemw12/btrfs-usage-monitor

or

    $ git clone https://github.com/willemw12/btrfs-usage-monitor
    $ cd btrfs-usage-monitor
    $ cargo build --release --out-dir=$HOME/bin


Usage
-----

    Print a warning if Btrfs filesystem data usage drops below the free limit percentage
    
    USAGE:
        btrfs-usage-monitor <path> <free-limit-percentage>
    
    ARGS:
        <path>                     Path to a subvolume or folder on the Btrfs filesystem
        <free-limit-percentage>    Maximum free filesystem data usage in percentage

Example
-------

    # btrfs-usage-monitor /mnt/btrfs 10
    WARNING /mnt/btrfs free: 752.58GiB (min: 681.47GiB), 9% (limit: 10%)


License
-------

GPL-3.0 or later


Links
-----

[Crates.io](https://crates.io/crates/btrfs-usage-monitor)  
[GitHub](https://github.com/willemw12/btrfs-usage-monitor)

