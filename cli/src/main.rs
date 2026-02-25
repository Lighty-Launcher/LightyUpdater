mod commands;
mod config_file;
mod daemon;
mod errors;
mod instance_lookup;
mod paths;
mod registry;
mod server;

use clap::{Parser, Subcommand};
use errors::CliResult;

#[derive(Parser)]
#[command(name = "lighty")]
#[command(about = "LightyUpdater CLI - Manage LightyUpdater instances", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install lighty to PATH
    Install,

    /// Create a new instance
    Create {
        /// Name of the instance
        #[arg(short, long)]
        name: String,

        /// Directory to create the instance in (defaults to current directory)
        #[arg(long)]
        dir: Option<String>,
    },

    /// Start an instance
    Start {
        /// Name of the instance to start
        #[arg(short, long)]
        name: Option<String>,
    },

    /// Stop an instance
    Stop {
        /// Name of the instance to stop
        #[arg(short, long)]
        name: Option<String>,

        /// Force kill the process if it doesn't stop gracefully
        #[arg(long)]
        force: bool,
    },

    /// Restart an instance
    Restart {
        /// Name of the instance to restart
        #[arg(short, long)]
        name: Option<String>,

        /// Force kill the process if it doesn't stop gracefully
        #[arg(long)]
        force: bool,
    },

    /// Show status of instances
    Status {
        /// Name of a specific instance
        #[arg(short, long)]
        name: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show logs of an instance
    Logs {
        /// Name of the instance
        #[arg(short, long)]
        name: Option<String>,

        /// Follow log output
        #[arg(short, long)]
        follow: bool,

        /// Watch log output (alias for --follow)
        #[arg(short, long)]
        watch: bool,

        /// Number of lines to show from the end
        #[arg(long, default_value = "50")]
        lines: usize,
    },

    /// List all instances
    List,

    /// Remove an instance from the registry
    Remove {
        /// Name of the instance to remove
        #[arg(short, long)]
        name: String,

        /// Also remove log files
        #[arg(long)]
        with_logs: bool,
    },

    /// Uninstall lighty CLI completely
    Uninstall {
        /// Force uninstall even if instances are running
        #[arg(long)]
        force: bool,
    },

    #[command(hide = true)]
    Serve {
        /// Path to config file
        #[arg(long)]
        config: String,
    },
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn run() -> CliResult<()> {
    let cli = Cli::parse();

    // Check if we're in server mode
    if let Commands::Serve { config } = &cli.command {
        // Run as server
        let config_path = std::path::PathBuf::from(config);
        server::run_server(config_path).await?;
        return Ok(());
    }

    // Check if we should suggest install
    match cli.command {
        Commands::Install => {
            // Don't suggest install when running install
        }
        _ => {
            commands::install::check_and_suggest_install();
        }
    }

    // Run CLI commands
    match cli.command {
        Commands::Serve { .. } => unreachable!(),
        Commands::Install => commands::install::execute(),
        Commands::Create { name, dir } => commands::create::execute(name, dir),
        Commands::Start { name } => commands::start::execute(name),
        Commands::Stop { name, force } => commands::stop::execute(name, force),
        Commands::Restart { name, force } => commands::restart::execute(name, force),
        Commands::Status { name, json } => commands::status::execute(name, json),
        Commands::Logs {
            name,
            follow,
            watch,
            lines,
        } => commands::logs::execute(name, follow || watch, lines),
        Commands::List => commands::list::execute(),
        Commands::Remove { name, with_logs } => commands::remove::execute(name, with_logs),
        Commands::Uninstall { force } => commands::uninstall::execute(force),
    }
}
