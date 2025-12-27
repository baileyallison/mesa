use std::fs::File;
use std::io::{self, BufRead};
use std::process::Command;

use crate::common::{GREEN, RED, YELLOW, RESET};

#[derive(Debug)]
pub struct HostInfo {
    pub distro_name: String,    // e.g., "Ubuntu" or "Rocky Linux"
    pub distro_version: String, // e.g., "22.04" or "9.3"
    pub kernel_version: String, // e.g., "5.15.0-91-generic"
}

pub fn print_host_info() {
    let info = get_host_info();
    
    println!("{}Distro:  {} {}{}", GREEN, info.distro_name, info.distro_version, RESET);
    println!("{}Kernel:  {}{}", GREEN, info.kernel_version, RESET);

}

fn get_host_info() -> HostInfo {
    // 1. Get Kernel via `uname -r`
    // We use Command here because parsing /proc/version text varies wildly between distros
    let kernel_output = Command::new("uname")
        .arg("-r")
        .output()
        .expect("Failed to execute uname");
    
    let kernel_version = String::from_utf8_lossy(&kernel_output.stdout)
        .trim()
        .to_string();

    // 2. Get OS Info via `/etc/os-release`
    let mut distro_name = String::from("Unknown");
    let mut distro_version = String::from("Unknown");

    if let Ok(file) = File::open("/etc/os-release") {
        let lines = io::BufReader::new(file).lines();
        for line in lines.flatten() {
            if let Some((key, value)) = line.split_once('=') {
                // Config values often come quoted like NAME="Ubuntu", so we trim quotes
                let clean_value = value.trim().trim_matches('"').to_string();
                
                match key {
                    "NAME" => distro_name = clean_value,
                    "VERSION_ID" => distro_version = clean_value,
                    _ => {}
                }
            }
        }
    }

    HostInfo {
        distro_name,
        distro_version,
        kernel_version,
    }
}
