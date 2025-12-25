# API Processing Flow

## GET /servers - Server List

```mermaid
sequenceDiagram
    participant Client
    participant Handler
    participant Cache
    participant Config

    Client->>Handler: GET /servers
    Handler->>Cache: get_all_servers()
    Cache-->>Handler: Vec<String> server names

    loop For each server
        Handler->>Cache: get_server_config(name)
        Cache-->>Handler: ServerConfig
        Handler->>Cache: get_last_update(name)
        Cache-->>Handler: RFC3339 timestamp
        Handler->>Handler: Build ServerInfo
    end

    Handler->>Client: 200 JSON ServerListResponse
```

## GET /{server}.json - Server Metadata

```mermaid
sequenceDiagram
    participant Client
    participant Handler
    participant Cache

    Client->>Handler: GET /server1.json
    Handler->>Handler: Strip .json suffix

    Handler->>Cache: get_server_config(server)
    Cache-->>Handler: ServerConfig

    alt Server disabled
        Handler->>Cache: get_all_servers()
        Cache-->>Handler: available list
        Handler->>Client: 404 ServerNotFound
    else Server enabled
        Handler->>Cache: get(server)

        alt Found
            Cache-->>Handler: VersionBuilder
            Handler->>Client: 200 JSON VersionBuilder
        else Not found
            Cache-->>Handler: None
            Handler->>Client: 404 ServerNotFound
        end
    end
```

## GET /{server}/{path} - File Serving

```mermaid
sequenceDiagram
    participant Client
    participant Handler
    participant Parser
    participant Resolver
    participant RAMCache
    participant Disk

    Client->>Handler: GET /server1/mods/mod.jar

    Handler->>Parser: parse_request_path()
    Parser->>Parser: Validate security
    Parser-->>Handler: ParsedRequest

    Handler->>Handler: Get version from cache

    Handler->>Resolver: resolve_file_path()
    Resolver->>Resolver: O(1) HashMap lookup
    Resolver-->>Handler: actual_path

    Handler->>RAMCache: try_serve_from_cache()

    alt In RAM cache
        RAMCache-->>Handler: Response (Bytes)
        Handler->>Client: 200 + file data
    else Not in cache
        Handler->>Disk: serve_from_disk()

        Disk->>Disk: Check file size

        alt Size < threshold
            Disk->>Disk: Load to memory
            Disk-->>Handler: Response (Vec<u8>)
        else Size >= threshold
            Disk->>Disk: Create stream
            Disk-->>Handler: Response (Stream)
        end

        Handler->>Client: 200 + file data
    end
```

## POST /rescan/{server} - Manual Rescan

```mermaid
sequenceDiagram
    participant Client
    participant Handler
    participant Cache
    participant Scanner

    Client->>Handler: POST /rescan/server1
    Handler->>Cache: force_rescan(server1)

    Cache->>Scanner: Scan server files
    Scanner-->>Cache: New VersionBuilder
    Cache->>Cache: Update cache
    Cache-->>Handler: Ok()

    Handler->>Client: 200 JSON success
```
