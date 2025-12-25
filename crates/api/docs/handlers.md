# Handlers Documentation

## list_servers

Lists all available servers.

**Route**: `GET /servers`

**Response**:
```json
{
  "servers": [
    {
      "name": "server1",
      "loader": "forge",
      "minecraft_version": "1.20.1",
      "url": "http://localhost:8080/server1.json",
      "last_update": "2024-01-15T10:30:00Z"
    }
  ]
}
```

**Flow**: Reads all server names from cache, builds ServerInfo for each with metadata.

---

## get_server_metadata

Returns complete metadata for a server.

**Route**: `GET /{server}.json`

**Example**: `/server1.json`

**Response**: Complete VersionBuilder JSON with all sections.

**Errors**:
- 404 if server does not exist or is disabled
- Returns list of available servers

---

## serve_file

Serves a specific file with intelligent caching.

**Route**: `GET /{server}/{path}`

**Examples**:
- `/server1/client/minecraft.jar`
- `/server1/mods/JEI-1.20.1.jar`
- `/server1/libraries/com/google/guava/31.0/guava-31.0.jar`

**Pipeline**:
1. Parse and validate path
2. Resolve URL to file path (O(1))
3. Attempt serving from RAM cache
4. Fallback to disk with streaming if large file

**Headers**:
- `Content-Type`: Automatically detected via mime_guess

---

## force_rescan

Triggers manual rescan of a server.

**Route**: `POST /rescan/{server}`

**Response**:
```json
{
  "status": "success",
  "message": "Server server1 rescanned successfully"
}
```

**Effect**: Launches complete rescan that detects changes and updates cache.
