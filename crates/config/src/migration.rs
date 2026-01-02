use super::errors::ConfigError;
use std::path::Path;
use std::sync::Arc;
use toml_edit::{Array, DocumentMut, Item, Table, Value};

type Result<T> = std::result::Result<T, ConfigError>;

/// Migrates config file to latest format if needed
pub async fn migrate_config_if_needed<P: AsRef<Path>>(
    path: P,
    events: Option<&Arc<lighty_events::EventBus>>,
) -> Result<()> {
    let content = tokio::fs::read_to_string(path.as_ref()).await?;
    let mut doc = content.parse::<DocumentMut>()?;
    let mut added_fields = Vec::new();

    migrate_server_section(&mut doc, &mut added_fields)?;
    migrate_cache_section(&mut doc, &mut added_fields)?;
    migrate_hot_reload_section(&mut doc, &mut added_fields)?;
    migrate_storage_section(&mut doc, &mut added_fields)?;
    migrate_cdn_section(&mut doc, &mut added_fields)?;
    migrate_cloudflare_section(&mut doc, &mut added_fields)?;
    migrate_servers_array(&mut doc, &mut added_fields)?;

    // Remove deprecated [metrics] section
    if doc.contains_key("metrics") {
        doc.remove("metrics");
        added_fields.push("removed deprecated [metrics] section".to_string());
    }

    // Only write if we added fields
    if !added_fields.is_empty() {
        tracing::info!("Migrating config with {} changes: {:?}", added_fields.len(), added_fields);
        tokio::fs::write(path.as_ref(), doc.to_string()).await?;
        tracing::debug!("Config file written after migration");

        if let Some(event_bus) = events {
            event_bus.emit(lighty_events::AppEvent::ConfigMigrated {
                added_fields: added_fields.clone(),
            });
        }
    } else {
        tracing::debug!("No migration needed");
    }

    Ok(())
}

fn migrate_server_section(
    doc: &mut DocumentMut,
    added_fields: &mut Vec<String>,
) -> Result<()> {
    // Ensure [server] section exists
    if !doc.contains_key("server") {
        let mut table = Table::new();
        table.set_implicit(true);
        doc["server"] = Item::Table(table);
        added_fields.push("server".to_string());
    }

    // Migrate allowed_origins from deprecated [security] section
    let existing_origins = doc
        .get("security")
        .and_then(|s| s.get("allowed_origins"))
        .cloned();

    // Ensure server fields
    let server = doc["server"]
        .as_table_mut()
        .ok_or_else(|| ConfigError::InvalidConfig("Invalid [server] section in config".to_string()))?;
    ensure_field(
        server,
        "host",
        Value::from("0.0.0.0"),
        added_fields,
    );
    ensure_field(server, "port", Value::from(8080), added_fields);
    ensure_field(
        server,
        "base_url",
        Value::from("http://localhost:8080"),
        added_fields,
    );
    ensure_field(
        server,
        "base_path",
        Value::from("updater"),
        added_fields,
    );
    ensure_field(server, "tcp_nodelay", Value::from(true), added_fields);
    ensure_field(server, "timeout_secs", Value::from(60), added_fields);
    ensure_field(
        server,
        "max_body_size_mb",
        Value::from(100),
        added_fields,
    );
    ensure_field(
        server,
        "streaming_threshold_mb",
        Value::from(100),
        added_fields,
    );
    ensure_field(
        server,
        "enable_compression",
        Value::from(true),
        added_fields,
    );

    // Add allowed_origins field
    if !server.contains_key("allowed_origins") {
        if let Some(origins) = existing_origins {
            server["allowed_origins"] = origins;
            added_fields.push("server.allowed_origins (migrated from security)".to_string());
        } else {
            let mut arr = Array::new();
            arr.push("*");
            server["allowed_origins"] = Item::Value(Value::Array(arr));
            added_fields.push("server.allowed_origins".to_string());
        }
    }

    // Remove deprecated [security] section
    if doc.contains_key("security") {
        doc.remove("security");
        added_fields.push("removed deprecated [security] section".to_string());
    }

    Ok(())
}

fn migrate_cache_section(
    doc: &mut DocumentMut,
    added_fields: &mut Vec<String>,
) -> Result<()> {
    // Ensure [cache] section
    if !doc.contains_key("cache") {
        let mut table = Table::new();
        table.set_implicit(true);
        doc["cache"] = Item::Table(table);
        added_fields.push("cache".to_string());
    }

    let cache = doc["cache"]
        .as_table_mut()
        .ok_or_else(|| ConfigError::InvalidConfig("Invalid [cache] section in config".to_string()))?;
    ensure_field(cache, "enabled", Value::from(true), added_fields);
    ensure_field(cache, "auto_scan", Value::from(true), added_fields);
    ensure_field(cache, "rescan_interval", Value::from(30), added_fields);
    ensure_field(
        cache,
        "max_memory_cache_gb",
        Value::from(0),
        added_fields,
    );
    ensure_field(
        cache,
        "checksum_buffer_size",
        Value::from(8192),
        added_fields,
    );
    ensure_field(
        cache,
        "hash_concurrency",
        Value::from(100),
        added_fields,
    );
    ensure_field(
        cache,
        "config_reload_channel_size",
        Value::from(100),
        added_fields,
    );

    // Migrate deprecated scan_batch_size to cache.batch.*
    let old_batch_size = if let Some(Item::Value(Value::Integer(val))) = cache.get("scan_batch_size") {
        Some(*val.value())
    } else {
        None
    };

    // Remove deprecated scan_batch_size
    if cache.contains_key("scan_batch_size") {
        cache.remove("scan_batch_size");
        added_fields.push("removed deprecated cache.scan_batch_size".to_string());
    }

    // Ensure [cache.batch] section
    if !cache.contains_key("batch") {
        let mut batch_table = Table::new();
        batch_table.set_implicit(true);
        cache["batch"] = Item::Table(batch_table);
        added_fields.push("cache.batch".to_string());
    }

    let batch = cache["batch"]
        .as_table_mut()
        .ok_or_else(|| ConfigError::InvalidConfig("Invalid [cache.batch] section in config".to_string()))?;
    let default_batch = old_batch_size.unwrap_or(100);
    ensure_field(batch, "client", Value::from(default_batch), added_fields);
    ensure_field(batch, "libraries", Value::from(default_batch), added_fields);
    ensure_field(batch, "mods", Value::from(default_batch), added_fields);
    ensure_field(batch, "natives", Value::from(default_batch), added_fields);
    ensure_field(batch, "assets", Value::from(default_batch), added_fields);

    Ok(())
}

fn migrate_hot_reload_section(
    doc: &mut DocumentMut,
    added_fields: &mut Vec<String>,
) -> Result<()> {
    // Extract old values from [cache] if they exist
    let old_config_debounce = if let Some(cache) = doc.get("cache") {
        cache.get("config_watch_debounce_ms")
            .and_then(|v| v.as_integer())
    } else {
        None
    };

    let old_files_debounce = if let Some(cache) = doc.get("cache") {
        cache.get("file_watcher_debounce_ms")
            .and_then(|v| v.as_integer())
    } else {
        None
    };

    // Ensure [hot-reload] section exists
    if !doc.contains_key("hot-reload") {
        let mut table = Table::new();
        table.set_implicit(true);
        doc["hot-reload"] = Item::Table(table);
        added_fields.push("hot-reload".to_string());
    }

    let hot_reload = doc["hot-reload"]
        .as_table_mut()
        .ok_or_else(|| ConfigError::InvalidConfig("Invalid [hot-reload] section in config".to_string()))?;

    // Ensure [hot-reload.config] section
    if !hot_reload.contains_key("config") {
        let mut config_table = Table::new();
        config_table.set_implicit(true);
        hot_reload["config"] = Item::Table(config_table);
        added_fields.push("hot-reload.config".to_string());
    }

    let config = hot_reload["config"]
        .as_table_mut()
        .ok_or_else(|| ConfigError::InvalidConfig("Invalid [hot-reload.config] section in config".to_string()))?;

    ensure_field(config, "enabled", Value::from(true), added_fields);

    // Use old value if exists, otherwise default to 300
    let config_debounce = old_config_debounce.unwrap_or(300);
    ensure_field(config, "debounce_ms", Value::from(config_debounce), added_fields);

    // Ensure [hot-reload.files] section
    if !hot_reload.contains_key("files") {
        let mut files_table = Table::new();
        files_table.set_implicit(true);
        hot_reload["files"] = Item::Table(files_table);
        added_fields.push("hot-reload.files".to_string());
    }

    let files = hot_reload["files"]
        .as_table_mut()
        .ok_or_else(|| ConfigError::InvalidConfig("Invalid [hot-reload.files] section in config".to_string()))?;

    ensure_field(files, "enabled", Value::from(true), added_fields);

    // Use old value if exists, otherwise default to 300
    let files_debounce = old_files_debounce.unwrap_or(300);
    ensure_field(files, "debounce_ms", Value::from(files_debounce), added_fields);

    // Remove old fields from [cache] if they exist
    if let Some(cache) = doc.get_mut("cache").and_then(|c| c.as_table_mut()) {
        if cache.contains_key("config_watch_debounce_ms") {
            cache.remove("config_watch_debounce_ms");
            added_fields.push("removed cache.config_watch_debounce_ms (migrated to hot-reload.config.debounce_ms)".to_string());
        }
        if cache.contains_key("file_watcher_debounce_ms") {
            cache.remove("file_watcher_debounce_ms");
            added_fields.push("removed cache.file_watcher_debounce_ms (migrated to hot-reload.files.debounce_ms)".to_string());
        }
    }

    Ok(())
}

fn migrate_storage_section(
    doc: &mut DocumentMut,
    added_fields: &mut Vec<String>,
) -> Result<()> {
    // Ensure [storage] section exists
    if !doc.contains_key("storage") {
        let mut table = Table::new();
        table.set_implicit(true);
        doc["storage"] = Item::Table(table);
        added_fields.push("storage".to_string());
    }

    let storage = doc["storage"]
        .as_table_mut()
        .ok_or_else(|| ConfigError::InvalidConfig("Invalid [storage] section in config".to_string()))?;

    ensure_field(storage, "backend", Value::from("local"), added_fields);
    ensure_field(storage, "keep_local_backup", Value::from(true), added_fields);
    ensure_field(storage, "auto_upload", Value::from(true), added_fields);

    // Ensure [storage.s3] section
    if !storage.contains_key("s3") {
        let mut s3_table = Table::new();
        s3_table.set_implicit(true);
        storage["s3"] = Item::Table(s3_table);
        added_fields.push("storage.s3".to_string());
    }

    let s3 = storage["s3"]
        .as_table_mut()
        .ok_or_else(|| ConfigError::InvalidConfig("Invalid [storage.s3] section in config".to_string()))?;

    ensure_field(s3, "enabled", Value::from(false), added_fields);
    ensure_field(s3, "endpoint_url", Value::from(""), added_fields);
    ensure_field(s3, "region", Value::from("auto"), added_fields);
    ensure_field(s3, "access_key_id", Value::from(""), added_fields);
    ensure_field(s3, "secret_access_key", Value::from(""), added_fields);
    ensure_field(s3, "bucket_name", Value::from("lighty-updater"), added_fields);
    ensure_field(s3, "public_url", Value::from(""), added_fields);
    ensure_field(s3, "bucket_prefix", Value::from(""), added_fields);

    Ok(())
}

fn migrate_cdn_section(
    doc: &mut DocumentMut,
    added_fields: &mut Vec<String>,
) -> Result<()> {
    // Ensure [cdn] section exists
    if !doc.contains_key("cdn") {
        let mut table = Table::new();
        table.set_implicit(true);
        doc["cdn"] = Item::Table(table);
        added_fields.push("cdn".to_string());
    }

    let cdn = doc["cdn"]
        .as_table_mut()
        .ok_or_else(|| ConfigError::InvalidConfig("Invalid [cdn] section in config".to_string()))?;

    ensure_field(cdn, "enabled", Value::from(false), added_fields);
    ensure_field(cdn, "provider", Value::from("cloudflare"), added_fields);
    ensure_field(cdn, "zone_id", Value::from(""), added_fields);
    ensure_field(cdn, "api_token", Value::from(""), added_fields);

    Ok(())
}

fn migrate_cloudflare_section(
    doc: &mut DocumentMut,
    added_fields: &mut Vec<String>,
) -> Result<()> {
    // Ensure [cloudflare] section exists
    if !doc.contains_key("cloudflare") {
        let mut table = Table::new();
        table.set_implicit(true);
        doc["cloudflare"] = Item::Table(table);
        added_fields.push("cloudflare".to_string());
    }

    let cloudflare = doc["cloudflare"]
        .as_table_mut()
        .ok_or_else(|| ConfigError::InvalidConfig("Invalid [cloudflare] section in config".to_string()))?;

    ensure_field(cloudflare, "enabled", Value::from(false), added_fields);
    ensure_field(cloudflare, "zone_id", Value::from(""), added_fields);
    ensure_field(cloudflare, "api_token", Value::from(""), added_fields);
    ensure_field(cloudflare, "base_url", Value::from(""), added_fields);

    // Remove deprecated purge_on_update field
    if cloudflare.contains_key("purge_on_update") {
        cloudflare.remove("purge_on_update");
        added_fields.push("removed deprecated cloudflare.purge_on_update".to_string());
    }

    Ok(())
}

fn migrate_servers_array(
    doc: &mut DocumentMut,
    added_fields: &mut Vec<String>,
) -> Result<()> {
    // Only migrate existing servers array - don't create empty one
    // This prevents "servers = []" from being added to the config file
    if let Some(servers_array) = doc
        .get_mut("servers")
        .and_then(|s| s.as_array_of_tables_mut())
    {
        // Migrate enabled field for each server
        for (idx, server_table) in servers_array.iter_mut().enumerate() {
            if !server_table.contains_key("enabled") {
                server_table.insert("enabled", Item::Value(Value::from(true)));
                added_fields.push(format!("servers[{}].enabled", idx));
            }
        }
    }

    Ok(())
}

fn ensure_field(
    table: &mut Table,
    key: &str,
    default_value: Value,
    added_fields: &mut Vec<String>,
) {
    if !table.contains_key(key) {
        table[key] = Item::Value(default_value);
        added_fields.push(key.to_string());
    }
}
