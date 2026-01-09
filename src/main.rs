mod api;
mod commands;
mod config;
mod sandbox;
mod utils;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "sapo-cli")]
#[command(author = "Salta")]
#[command(version = "1.0.0")]
#[command(about = "Pre-install protection for npm packages", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show current status
    Status,

    /// Scan a package for threats
    Scan {
        /// Package name to scan
        package: String,
    },

    /// Add a package to the trusted list
    Trust {
        /// Package name to trust
        package: String,
    },

    /// Remove a package from the trusted list
    Untrust {
        /// Package name to untrust
        package: String,
    },

    /// Show all trusted packages
    Trusted,

    /// Temporarily disable protection
    Disable,

    /// Re-enable protection
    Enable,

    /// Sync Pro features from server
    Sync,

    /// Open login page to link device
    Login,

    /// Open pricing page
    Upgrade,

    /// Update to latest version
    Update,

    /// Uninstall Sapo
    Uninstall,

    /// Wrap a package manager command (internal use)
    Wrap {
        /// Package manager (npm, pnpm, yarn, bun)
        manager: String,
        /// Arguments to pass to package manager
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },

    /// Runtime monitoring commands (Layer 4)
    Monitor {
        #[command(subcommand)]
        action: MonitorAction,
    },
}

#[derive(Subcommand)]
enum MonitorAction {
    /// Show runtime monitoring status
    Status,

    /// Enable runtime monitoring for install commands
    Enable,

    /// Disable runtime monitoring
    Disable,

    /// Toggle runtime monitoring on/off
    Toggle,

    /// Show recent runtime threats
    Threats {
        /// Number of threats to show
        #[arg(short, long, default_value = "20")]
        count: usize,
    },

    /// Clear the threat log
    Clear,
}

fn main() {
    let cli = Cli::parse();

    // Initialize config on first run
    config::init();

    match cli.command {
        Commands::Status => commands::status::run(),
        Commands::Scan { package } => {
            commands::scan::run(&package, true);
        }
        Commands::Trust { package } => commands::trust::add(&package),
        Commands::Untrust { package } => commands::trust::remove(&package),
        Commands::Trusted => commands::trust::list(),
        Commands::Disable => commands::toggle::disable(),
        Commands::Enable => commands::toggle::enable(),
        Commands::Sync => commands::sync::run(),
        Commands::Login => commands::login::run(),
        Commands::Upgrade => commands::upgrade::run(),
        Commands::Update => commands::update::run(),
        Commands::Uninstall => commands::uninstall::run(),
        Commands::Wrap { manager, args } => commands::wrap::run(&manager, &args),
        Commands::Monitor { action } => match action {
            MonitorAction::Status => commands::monitor::show_status(),
            MonitorAction::Enable => commands::monitor::enable(),
            MonitorAction::Disable => commands::monitor::disable(),
            MonitorAction::Toggle => commands::monitor::toggle(),
            MonitorAction::Threats { count } => commands::monitor::show_threats(count),
            MonitorAction::Clear => commands::monitor::clear_log(),
        },
    }
}
