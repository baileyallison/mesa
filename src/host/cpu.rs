use std::fs;
use std::io::{self, BufRead};
use std::path::Path;
use std::process::Command;

use crate::common::{GREEN, RED, YELLOW, RESET};

/// 1. Check active tuned-adm profile
/// Executes the shell command `tuned-adm active`.
pub fn check_tuned_profile() {
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

/// 2.. Verify CPU Sleep States (C-States)
/// Checks kernel cmdline for standard disable flags.
pub fn check_cpu_cstates() {
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
