use std::fs;
use std::io::{self, BufRead};
use std::path::Path;
use std::process::Command;

use crate::common::{GREEN, RED, YELLOW, RESET};

/// 1. Verify Swap Usage
/// Reads /proc/meminfo to calculate SwapTotal and SwapFree.
pub fn check_swap_status() {
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
