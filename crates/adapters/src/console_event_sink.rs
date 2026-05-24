use colored::Colorize;
use lighty_events::{AppEvent, EventSink};

pub struct ConsoleEventSink;

impl ConsoleEventSink {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ConsoleEventSink {
    fn default() -> Self {
        Self::new()
    }
}

impl EventSink for ConsoleEventSink {
    fn handle(&self, event: &AppEvent) {
        match event {
            AppEvent::Starting { version } => {
                println!("\n{}", "-----------------------------------------".bright_black());
                println!("  {}", "LightyUpdater - Distribution Server".white().bold());
                println!("  {} {}", "Version".dimmed(), version.cyan());
                println!("{}\n", "-----------------------------------------".bright_black());
            }
            AppEvent::Ready { addr, base_url } => {
                println!("{}", "-----------------------------------------".green());
                println!("  {} {}", "Server".white(), addr.cyan());
                println!("  {} {}", "URL   ".white(), base_url.blue());
                println!("{}\n", "-----------------------------------------".green());
            }
            AppEvent::Shutdown => {
                println!("\n{}", "Server shutting down".red());
            }
            AppEvent::ConfigLoading { path } => {
                println!("  {} {}", "Loading config".dimmed(), path.cyan());
            }
            AppEvent::ConfigLoaded { servers_count } => {
                if *servers_count == 0 {
                    println!("  {} No servers configured", "!".yellow());
                } else {
                    println!("  {} {} server(s)", "ok".green(), servers_count.to_string().cyan());
                }
            }
            AppEvent::ConfigCreated { path } => {
                tracing::warn!("Configuration file not found");
                tracing::info!("Created default configuration at: {}", path);
            }
            AppEvent::ConfigMigrated { added_fields } => {
                if !added_fields.is_empty() {
                    println!(
                        "  {} Config updated: added {}",
                        "->".blue(),
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
            AppEvent::ServerFolderInit { .. }
            | AppEvent::ServerFolderCreated { .. }
            | AppEvent::AllServersInitialized
            | AppEvent::ScanStarted { .. }
            | AppEvent::ScanCompleted { .. }
            | AppEvent::CacheUnchanged { .. } => {}
            AppEvent::InitialScanStarted => {
                println!("  {} Scanning servers...", "->".dimmed());
            }
            AppEvent::CacheNew { server } => {
                println!("  {} Cached {}", "ok".green(), server.cyan());
            }
            AppEvent::CacheUpdated { server, changes } => {
                if !changes.is_empty() {
                    println!(
                        "  {} Updated {} ({})",
                        "->".blue(),
                        server.cyan(),
                        changes.join(", ").dimmed()
                    );
                }
            }
            AppEvent::NewServerDetected { name } => {
                println!("  {} New server: {}", "+".green(), name.cyan());
            }
            AppEvent::ServerRemoved { name } => {
                println!("  {} Removed: {}", "-".red(), name.cyan());
            }
            AppEvent::AutoScanEnabled { interval } => {
                println!("  {} Auto-scan {}s", "->".blue(), interval.to_string().cyan());
            }
            AppEvent::ContinuousScanEnabled => {
                println!("  {} Continuous scan", "->".blue());
            }
            AppEvent::CdnPurgeRequested { server, urls } => {
                println!(
                    "  {} CDN purge {} ({} url(s))",
                    "->".blue(),
                    server.cyan(),
                    urls.len().to_string().dimmed()
                );
            }
            AppEvent::CdnPurgeCompleted { server, ok, error } => {
                if *ok {
                    println!("  {} CDN purged {}", "ok".green(), server.cyan());
                } else {
                    tracing::error!(
                        "CDN purge failed for {}: {}",
                        server,
                        error.as_deref().unwrap_or("unknown error")
                    );
                }
            }
            AppEvent::CloudflarePurgeRequested { server } => {
                println!("  {} Cloudflare purge {}", "->".blue(), server.cyan());
            }
            AppEvent::CloudflarePurgeCompleted { server, ok, error } => {
                if *ok {
                    println!("  {} Cloudflare purged {}", "ok".green(), server.cyan());
                } else {
                    tracing::error!(
                        "Cloudflare purge failed for {}: {}",
                        server,
                        error.as_deref().unwrap_or("unknown error")
                    );
                }
            }
            AppEvent::Error { context, error } => {
                tracing::error!("{}: {}", context, error);
            }
        }
    }
}
