use std::collections::HashSet;
use proc_mounts::MountIter;
use std::io;

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

    for mount in MountIter::new()?.map(|m| m.unwrap()) {
        if virtual_filesystems.contains(mount.fstype.as_str()) {
            continue;
        }
        if mount.dest.starts_with("/snap/") && mount.fstype == "squashfs" {
            continue;
        }
        println!("{} on {} type {}", mount.source.display(), mount.dest.display(), mount.fstype);
    }

    Ok(())
}
