use super::models::{AppEvent, EventBus};
use std::sync::Arc;
use colored::Colorize;

impl EventBus {
    pub fn new(silent_mode: bool) -> Arc<Self> {
        Arc::new(Self { silent_mode })
    }

    pub fn emit(&self, event: AppEvent) {
        match event {
            // Application lifecycle
            AppEvent::Starting => {
                println!("\n{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black());
                println!("  {}", "LightyUpdater - Distribution Server".white().bold());
                println!("  {} {}", "Version".dimmed(), env!("CARGO_PKG_VERSION").cyan());
                println!("{}\n", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black());
            }
            AppEvent::Ready { addr, base_url } => {
                println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".green());
                println!("  {} {}", "Server".white(), addr.cyan());
                println!("  {} {}", "URL   ".white(), base_url.blue());
                println!("{}\n", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".green());
            }
            AppEvent::Shutdown => {
                println!("\n{}", "Server shutting down".red());
            }

            // Configuration
            AppEvent::ConfigLoading { path } => {
                println!("  {} {}", "Loading config".dimmed(), path.cyan());
            }
            AppEvent::ConfigLoaded { servers_count } => {
                if servers_count == 0 {
                    println!("  {} No servers configured", "⚠".yellow());
                } else {
                    println!("  {} {} server(s)", "✓".green(), servers_count.to_string().cyan());
                }
            }
            AppEvent::ConfigCreated { path } => {
                tracing::warn!("Configuration file not found");
                tracing::info!("Created default configuration at: {}", path);
            }
            AppEvent::ConfigMigrated { added_fields } => {
                if !added_fields.is_empty() {
                    println!("  {} Config updated: added {}",
                        "↻".blue(),
                        added_fields.join(", ").dimmed()
                    );
                }
            }
            AppEvent::ConfigReloaded => {
                tracing::info!("Configuration reloaded successfully");
            }
            AppEvent::ConfigError { error } => {
                tracing::error!("Configuration error: {}", error);
            }

            // Server initialization
            AppEvent::ServerFolderInit { .. } | AppEvent::ServerFolderCreated { .. } => {
                // Silent - reduce verbosity
            }
            AppEvent::AllServersInitialized => {
                // Silent
            }

            // Scanning
            AppEvent::ScanStarted { .. } | AppEvent::ScanCompleted { .. } => {
                // Silent during scans
            }
            AppEvent::InitialScanStarted => {
                println!("  {} Scanning servers...", "→".dimmed());
            }

            // Cache events
            AppEvent::CacheNew { server } => {
                println!("  {} Cached {}", "✓".green(), server.cyan());
            }
            AppEvent::CacheUpdated { server, changes } => {
                if !changes.is_empty() {
                    println!("  {} Updated {} ({})", "↻".blue(), server.cyan(), changes.join(", ").dimmed());
                }
            }
            AppEvent::CacheUnchanged { .. } => {
                // Silent
            }

            // Server discovery
            AppEvent::NewServerDetected { name } => {
                println!("  {} New server: {}", "+".green(), name.cyan());
            }
            AppEvent::ServerRemoved { name } => {
                println!("  {} Removed: {}", "-".red(), name.cyan());
            }

            // Auto-scan
            AppEvent::AutoScanEnabled { interval } => {
                println!("  {} Auto-scan {}s", "↻".blue(), interval.to_string().cyan());
            }
            AppEvent::ContinuousScanEnabled => {
                println!("  {} Continuous scan", "↻".blue());
            }

            // Errors
            AppEvent::Error { context, error } => {
                tracing::error!("{}: {}", context, error);
            }
        }
    }
}
