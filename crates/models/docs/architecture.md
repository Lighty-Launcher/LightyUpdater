# Data Model Architecture

## Structure Hierarchy

```mermaid
classDiagram
    class VersionBuilder {
        +MainClass main_class
        +JavaVersion java_version
        +Arguments arguments
        +Vec~Library~ libraries
        +Vec~Mod~ mods
        +Option~Vec~Native~~ natives
        +Option~Client~ client
        +Vec~Asset~ assets
        +HashMap url_to_path_map
        +build_url_map()
        +add_url_mapping()
        +remove_url_mapping()
    }

    class MainClass {
        +String main_class
    }

    class JavaVersion {
        +u8 major_version
    }

    class Arguments {
        +Vec~String~ game
        +Vec~String~ jvm
    }

    class Library {
        +String name
        +Option~String~ url
        +Option~String~ path
        +Option~String~ sha1
        +Option~u64~ size
    }

    class Mod {
        +String name
        +Option~String~ url
        +Option~String~ path
        +Option~String~ sha1
        +Option~u64~ size
    }

    class Native {
        +String name
        +String url
        +String path
        +String sha1
        +u64 size
        +String os
    }

    class Client {
        +String name
        +String url
        +String path
        +String sha1
        +u64 size
    }

    class Asset {
        +String hash
        +u64 size
        +Option~String~ url
        +Option~String~ path
    }

    VersionBuilder --> MainClass
    VersionBuilder --> JavaVersion
    VersionBuilder --> Arguments
    VersionBuilder --> Library
    VersionBuilder --> Mod
    VersionBuilder --> Native
    VersionBuilder --> Client
    VersionBuilder --> Asset
```

## Format JSON

```json
{
  "main_class": {
    "main_class": "net.minecraft.client.main.Main"
  },
  "java_version": {
    "major_version": 17
  },
  "arguments": {
    "game": ["--username", "${auth_player_name}"],
    "jvm": ["-Xmx2G", "-XX:+UnlockExperimentalVMOptions"]
  },
  "libraries": [
    {
      "name": "com.google.guava:guava:31.0",
      "url": "http://localhost/server1/libraries/com/google/guava/31.0/guava-31.0.jar",
      "path": "com/google/guava/31.0/guava-31.0.jar",
      "sha1": "abc123...",
      "size": 2784000
    }
  ],
  "mods": [
    {
      "name": "JEI",
      "url": "http://localhost/server1/mods/JEI-1.20.1.jar",
      "path": "JEI-1.20.1.jar",
      "sha1": "def456...",
      "size": 1024000
    }
  ]
}
```

## Optional Fields

Some fields are Option to support different use cases:

**Optional url/path**: Maven libraries may have path but no URL if not hosted.

**Optional natives**: Servers without native components omit the field.

**Optional client**: Servers without client JAR (dedicated servers).

**Optional sha1/size**: Files without integrity verification.
