use colored::Colorize;

#[allow(dead_code)]
pub fn print_header() {
    println!();
    println!(
        "{}",
        "  +--------------------------------------------+".green()
    );
    println!(
        "{}",
        "  |  Sapo CLI v1.0.0                           |".green()
    );
    println!(
        "{}",
        "  |  Pre-install protection for npm packages   |".green()
    );
    println!(
        "{}",
        "  +--------------------------------------------+".green()
    );
    println!();
}

pub fn print_ok(msg: &str) {
    println!("  {} {}", "[OK]".green(), msg);
}

pub fn print_info(msg: &str) {
    println!("  {} {}", "[>]".cyan(), msg);
}

pub fn print_warning(msg: &str) {
    println!("  {} {}", "[!]".yellow(), msg);
}

pub fn print_error(msg: &str) {
    println!("  {} {}", "[X]".red(), msg);
}

pub fn print_blocked(msg: &str) {
    println!("  {} {}", "[X] BLOCKED:".red().bold(), msg);
}

pub fn print_detail(msg: &str) {
    println!("     {} {}", "|-".bright_black(), msg.bright_black());
}

#[allow(dead_code)]
pub fn print_tree_end(msg: &str) {
    println!("  {} {}", "+-".bright_black(), msg.bright_black());
}

pub fn print_section_header(title: &str) {
    println!("  {}", title.cyan().bold());
    println!("  {}", "─".repeat(title.len()).cyan());
}
