use std::path::Path;
use std::sync::Arc;
use toml_edit::{Array, DocumentMut, Item, Table, Value};

/// Migrates config file to latest format if needed
pub async fn migrate_config_if_needed<P: AsRef<Path>>(
    path: P,
    events: Option<&Arc<lighty_events::EventBus>>,
) -> anyhow::Result<()> {
    let content = tokio::fs::read_to_string(path.as_ref()).await?;
    let mut doc = content.parse::<DocumentMut>()?;
    let mut added_fields = Vec::new();

    migrate_server_section(&mut doc, &mut added_fields)?;
    migrate_cache_section(&mut doc, &mut added_fields)?;
    migrate_servers_array(&mut doc, &mut added_fields)?;

    // Remove deprecated [metrics] section
    if doc.contains_key("metrics") {
        doc.remove("metrics");
        added_fields.push("removed deprecated [metrics] section".to_string());
    }

    // Only write if we added fields
    if !added_fields.is_empty() {
        tokio::fs::write(path.as_ref(), doc.to_string()).await?;

        if let Some(event_bus) = events {
            event_bus.emit(lighty_events::AppEvent::ConfigMigrated {
                added_fields: added_fields.clone(),
            });
        }
    }

    Ok(())
}

fn migrate_server_section(
    doc: &mut DocumentMut,
    added_fields: &mut Vec<String>,
) -> anyhow::Result<()> {
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
        .ok_or_else(|| anyhow::anyhow!("Invalid [server] section in config"))?;
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
) -> anyhow::Result<()> {
    // Ensure [cache] section
    if !doc.contains_key("cache") {
        let mut table = Table::new();
        table.set_implicit(true);
        doc["cache"] = Item::Table(table);
        added_fields.push("cache".to_string());
    }

    let cache = doc["cache"]
        .as_table_mut()
        .ok_or_else(|| anyhow::anyhow!("Invalid [cache] section in config"))?;
    ensure_field(cache, "enabled", Value::from(true), added_fields);
    ensure_field(cache, "auto_scan", Value::from(true), added_fields);
    ensure_field(cache, "rescan_interval", Value::from(30), added_fields);
    ensure_field(
        cache,
        "config_watch_debounce_ms",
        Value::from(500),
        added_fields,
    );
    ensure_field(
        cache,
        "max_memory_cache_gb",
        Value::from(0),
        added_fields,
    );
    ensure_field(
        cache,
        "file_watcher_debounce_ms",
        Value::from(500),
        added_fields,
    );
    ensure_field(
        cache,
        "checksum_buffer_size",
        Value::from(8192),
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
        .ok_or_else(|| anyhow::anyhow!("Invalid [cache.batch] section in config"))?;
    let default_batch = old_batch_size.unwrap_or(100);
    ensure_field(batch, "client", Value::from(default_batch), added_fields);
    ensure_field(batch, "libraries", Value::from(default_batch), added_fields);
    ensure_field(batch, "mods", Value::from(default_batch), added_fields);
    ensure_field(batch, "natives", Value::from(default_batch), added_fields);
    ensure_field(batch, "assets", Value::from(default_batch), added_fields);

    Ok(())
}

fn migrate_servers_array(
    doc: &mut DocumentMut,
    added_fields: &mut Vec<String>,
) -> anyhow::Result<()> {
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
