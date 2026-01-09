use crate::utils::*;
use std::env;

pub fn disable() {
    // Note: This only works in the same process
    // The shell wrapper will check SAPO_DISABLED env var
    env::set_var("SAPO_DISABLED", "1");
    print_warning("Sapo paused for this session");
    println!("  {}", "Note: Run in your shell: export SAPO_DISABLED=1".bright_black());
}

pub fn enable() {
    env::remove_var("SAPO_DISABLED");
    print_ok("Sapo active");
    println!("  {}", "Note: Run in your shell: unset SAPO_DISABLED".bright_black());
}

use colored::Colorize;
