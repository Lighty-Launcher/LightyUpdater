mod commands;
mod errors;
mod process;
mod server;
mod state;
mod ui;

use clap::{Parser, Subcommand};
use errors::CliResult;
use std::path::PathBuf;
use ui::format::OutputFormat;

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
        name: String,

        /// Directory to create the instance in (defaults to current directory)
        #[arg(long)]
        dir: Option<String>,
    },

    /// Start an instance
    Start {
        /// Name of the instance (defaults to current directory)
        name: Option<String>,
    },

    /// Stop an instance
    Stop {
        /// Name of the instance (defaults to current directory)
        name: Option<String>,

        /// Force kill the process if it doesn't stop gracefully
        #[arg(long)]
        force: bool,
    },

    /// Restart an instance
    Restart {
        /// Name of the instance (defaults to current directory)
        name: Option<String>,

        /// Force kill the process if it doesn't stop gracefully
        #[arg(long)]
        force: bool,
    },

    /// Show details of an instance
    Describe {
        /// Name of the instance (defaults to current directory)
        name: Option<String>,

        /// Output format
        #[arg(short = 'o', long)]
        output: Option<OutputFormat>,
    },

    /// Show logs of an instance
    Logs {
        /// Name of the instance (defaults to current directory)
        name: Option<String>,

        /// Stream new log lines as they arrive
        #[arg(short, long)]
        follow: bool,

        /// Number of lines to show from the end
        #[arg(long, default_value = "50")]
        tail: usize,
    },

    /// List all instances
    Get {
        /// Output format
        #[arg(short = 'o', long)]
        output: Option<OutputFormat>,

        /// Re-render as instances change state
        #[arg(short = 'w', long)]
        watch: bool,
    },

    /// Remove an instance from the registry
    Remove {
        /// Name of the instance to remove
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
    if let Err(error) = run().await {
        eprintln!("Error: {}", error);
        std::process::exit(1);
    }
}

async fn run() -> CliResult<()> {
    let cli = Cli::parse();

    if let Commands::Serve { config } = &cli.command {
        let config_path = PathBuf::from(config);
        server::run_server(config_path).await?;
        return Ok(());
    }

    match cli.command {
        Commands::Install => {}
        _ => commands::install::check_and_suggest_install(),
    }

    match cli.command {
        Commands::Serve { .. } => unreachable!(),
        Commands::Install => commands::install::execute(),
        Commands::Create { name, dir } => commands::create::execute(name, dir),
        Commands::Start { name } => commands::start::execute(name),
        Commands::Stop { name, force } => commands::stop::execute(name, force),
        Commands::Restart { name, force } => commands::restart::execute(name, force),
        Commands::Describe { name, output } => commands::describe::execute(name, output),
        Commands::Logs { name, follow, tail } => commands::logs::execute(name, follow, tail),
        Commands::Get { output, watch } => commands::get::execute(output, watch),
        Commands::Remove { name, with_logs } => commands::remove::execute(name, with_logs),
        Commands::Uninstall { force } => commands::uninstall::execute(force),
    }
}
