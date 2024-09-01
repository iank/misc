use proc_mounts::MountIter;
use std::collections::HashSet;
use std::io;
use tabled::{builder::Builder, settings::Style};

fn main() -> io::Result<()> {
    // overlay is debatable here
    let virtual_filesystems: HashSet<&str> = [
        "binfmt_misc",
        "cgroup",
        "cgroup2",
        "devpts",
        "devtmpfs",
        "overlay",
        "proc",
        "sysfs",
        "tmpfs",
        "nsfs",
        "rpc_pipefs",
        "securityfs",
        "pstore",
        "efivarfs",
        "bpf",
        "autofs",
        "mqueue",
        "hugetlbfs",
        "debugfs",
        "tracefs",
        "fusectl",
        "configfs",
        "ramfs",
        "nfsd",
    ]
    .iter()
    .copied()
    .collect();

    let mut builder = Builder::new();

    for mount in MountIter::new()?.map(|m| m.unwrap()) {
        if virtual_filesystems.contains(mount.fstype.as_str()) {
            continue;
        }
        if mount.dest.starts_with("/snap/") && mount.fstype == "squashfs" {
            continue;
        }
        builder.push_record([
            mount.source.display().to_string(),
            mount.dest.display().to_string(),
            mount.fstype,
        ]);
    }

    let table = builder.build().with(Style::blank()).to_string();
    println!("{}", table);

    Ok(())
}
