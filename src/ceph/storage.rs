use std::fs;
use std::io::{self, BufRead};
use std::path::Path;
use std::process::Command;

use crate::common::{GREEN, RED, YELLOW, RESET};

// check if udev rule exists and is enabled
pub fn check_udev_rule_exists() {
    println!("Checking if udev rule exists");
    let udev_path = ("/etc/udev/rules.d/99-ceph-write-through.rules");
    if Path::new(udev_path).exists() {
        println!("{}OK:    udev rule exists{}", GREEN, RESET);
        } else {
        println!("{}FAIL:  udev rule doesn't exist{}", RED, RESET);
        }
}

// check if each drive has write cache enabled or disabled
// checks /sys/block to find sd* and nvme* devices and checks their write_cache mode.
pub fn check_disk_write_cache() {
    println!("Checking Disk Write Caches (Expecting: 'write through' or disabled)...");

    let sys_block = Path::new("/sys/block");
    if let Ok(entries) = fs::read_dir(sys_block) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();

            // filter for sdX (SATA/SAS)
            if name.starts_with("sd") {
                // construct path: /sys/block/<dev>/queue/write_cache
                let cache_path = entry.path().join("queue/write_cache");

                if cache_path.exists() {
                    match fs::read_to_string(&cache_path) {
                        Ok(contents) => {
                            let mode = contents.trim();
                            if mode == "write back" {
                                println!("{}FAIL: {} is using '{}'{}", RED, name, mode, RESET);
                            } else {
                                println!("{}OK:   {} is using '{}', disabled{}", GREEN, name, mode, RESET);
                            }
                        },
                        Err(_) => println!("{}WARN: Could not read cache for {}{}", YELLOW, name, RESET),
                    }
                } else {
                    println!("{}INFO: {} does not support software toggle for write cache{}", YELLOW, name, RESET);
                }
            }
        }
    } else {
        println!("{}ERROR: Could not read /sys/block{}", RED, RESET);
    }
}
