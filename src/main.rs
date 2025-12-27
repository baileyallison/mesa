// ANSI Color codes for output readability
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const RESET: &str = "\x1b[0m";

mod common;
mod distro;
mod host;
mod ceph;

fn main() {
    println!("--- Starting Linux Tuning Check ---\n");

    // 2. Call the functions using the filename::function_name syntax

    distro::distro_check::print_host_info();
    println!("-----------------------------------");

    ceph::storage::check_udev_rule_exists();
    println!("-----------------------------------");

    ceph::storage::check_disk_write_cache();
    println!("-----------------------------------");

    host::cpu::check_tuned_profile();
    println!("-----------------------------------");

    host::memory::check_swap_status();
    println!("-----------------------------------");

    host::cpu::check_cpu_cstates();

    println!("\n--- Check Complete ---");
}
