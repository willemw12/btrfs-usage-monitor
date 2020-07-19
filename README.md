Btrfs usage monitor
===================

A simple Btrfs usage monitor.

There is also a version written in [Go](https://github.com/willemw12/btrfs-usage-monitor-go).


Feature
-------

- Print a warning if Btrfs data usage drops below the free limit percentage.


Installation
------------

To install from crates.io to $HOME/bin/, run, for example:

    $ cargo install --root $HOME/ btrfs-usage-monitor

To install from GitHub to $HOME/bin/, for example the latest commit, run:

    $ cargo install --root $HOME/ --git https://github.com/willemw12/btrfs-usage-monitor

or

    $ git clone https://github.com/willemw12/btrfs-usage-monitor
    $ cd btrfs-usage-monitor
    $ cargo build --release --out-dir=$HOME/bin/


Usage
-----

    # btrfs-usage-monitor /mnt/btrfs 10
    WARNING /mnt/btrfs free: 752.58GiB (min: 681.47GiB), 9% (limit: 10%)


License
-------

GPL-3.0 or later


Link
----

[Crates.io](https://crates.io/crates/btrfs-usage-monitor)
[GitHub](https://github.com/willemw12/btrfs-usage-monitor)

