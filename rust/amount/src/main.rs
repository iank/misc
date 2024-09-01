use amount::parser;
use std::collections::HashSet;
use std::fs::read_to_string;

fn main() {
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

    for line in read_to_string("/proc/mounts").unwrap().lines() {
        match parser::parse_line(line) {
            Ok((_, m)) => {
                if !virtual_filesystems.contains(m.filesystem.as_str())
                    && !(m.mount_point.starts_with("/snap")
                        && m.filesystem == "squashfs")
                {
                    println!("{m}");
                }
            }
            Err(_) => {
                println!("Failed to parse line {line}");
            }
        }
    }
}
