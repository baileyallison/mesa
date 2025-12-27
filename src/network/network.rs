use std::fs;
use std::io::{self, BufRead};
use std::path::Path;
use std::process::Command;

// ANSI Color codes for output readability
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const RESET: &str = "\x1b[0m";

fn main() {
    println!("--- Starting Linux Tuning Check ---\n");

    check_disk_write_cache();
    println!("-----------------------------------");
    check_tuned_profile();
    println!("-----------------------------------");
    check_swap_status();
    println!("-----------------------------------");
    check_cpu_cstates();

    println!("\n--- Check Complete ---");
}

/// 1. Check Disk Write Cache
/// Iterates /sys/block to find sd* and nvme* devices and checks their write_cache mode.
fn check_disk_write_cache() {
    println!("Checking Disk Write Caches (Expecting: 'write through' or disabled)...");

    let sys_block = Path::new("/sys/block");
    if let Ok(entries) = fs::read_dir(sys_block) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();

            // Filter for sdX (SATA/SAS) and nvmeXn1 (NVMe)
            if name.starts_with("sd") || name.starts_with("nvme") {
                // Construct path: /sys/block/<dev>/queue/write_cache
                let cache_path = entry.path().join("queue/write_cache");

                if cache_path.exists() {
                    match fs::read_to_string(&cache_path) {
                        Ok(contents) => {
                            let mode = contents.trim();
                            // 'write back' implies cache is ON (risky for ZFS without battery backup)
                            // 'write through' implies cache is OFF/Safe
                            if mode == "write back" {
                                println!("{}FAIL: {} is using '{}'{}", RED, name, mode, RESET);
                            } else {
                                println!("{}OK:   {} is using '{}'{}", GREEN, name, mode, RESET);
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

/// 2. Check active tuned-adm profile
/// Executes the shell command `tuned-adm active`.
fn check_tuned_profile() {
    println!("Checking tuned-adm profile...");

    match Command::new("tuned-adm").arg("active").output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let trimmed = stdout.trim();
            if trimmed.contains("Current active profile") {
                 println!("{}{}{}", GREEN, trimmed, RESET);
            } else {
                 println!("{}WARN: output unexpected: {}{}", YELLOW, trimmed, RESET);
            }
        },
        Err(_) => println!("{}FAIL: 'tuned-adm' command not found or failed to run. Is it installed?{}", RED, RESET),
    }
}

/// 3. Verify Swap Usage
/// Reads /proc/meminfo to calculate SwapTotal and SwapFree.
fn check_swap_status() {
    println!("Checking Swap Status (Expecting: Swap Used = 0)...");

    if let Ok(file) = fs::File::open("/proc/meminfo") {
        let reader = io::BufReader::new(file);
        let mut swap_total = 0;
        let mut swap_free = 0;

        for line in reader.lines().flatten() {
            if line.starts_with("SwapTotal:") {
                swap_total = parse_kb_value(&line);
            } else if line.starts_with("SwapFree:") {
                swap_free = parse_kb_value(&line);
            }
        }

        let swap_used = swap_total - swap_free;

        if swap_total == 0 {
            println!("{}OK: Swap is disabled (Total: 0 kB){}", GREEN, RESET);
        } else if swap_used > 0 {
            println!("{}FAIL: Swap is IN USE. Used: {} kB / Total: {} kB{}", RED, swap_used, swap_total, RESET);
        } else {
            println!("{}OK: Swap exists but is empty. (Total: {} kB){}", GREEN, swap_total, RESET);
        }

    } else {
        println!("{}ERROR: Could not read /proc/meminfo{}", RED, RESET);
    }
}

/// Helper to parse "Key:       1234 kB" from meminfo
fn parse_kb_value(line: &str) -> u64 {
    // Split by whitespace and take the second element
    line.split_whitespace()
        .nth(1)
        .unwrap_or("0")
        .parse()
        .unwrap_or(0)
}

/// 4. Verify CPU Sleep States (C-States)
/// Checks kernel cmdline for standard disable flags.
fn check_cpu_cstates() {
    println!("Checking CPU C-States configuration...");

    match fs::read_to_string("/proc/cmdline") {
        Ok(cmdline) => {
            let checks = ["intel_idle.max_cstate=0", "processor.max_cstate=1"];
            let mut found = false;

            for check in checks.iter() {
                if cmdline.contains(check) {
                    println!("{}OK: Found boot parameter '{}' (C-states restricted){}", GREEN, check, RESET);
                    found = true;
                }
            }

            if !found {
                println!("{}WARN: No C-state restriction flags found in kernel boot parameters.{}", YELLOW, RESET);
                println!("      Current cmdline: {}", cmdline.trim());
            }
        },
        Err(_) => println!("{}ERROR: Could not read /proc/cmdline{}", RED, RESET),
    }
}
