# Data Model Architecture

## Structure Hierarchy

```mermaid
graph TB
    VB[VersionBuilder] --> MC[MainClass]
    VB --> JV[JavaVersion]
    VB --> Args[Arguments]
    VB --> Lib[Libraries]
    VB --> Mods[Mods]
    VB --> Nat[Natives]
    VB --> Cli[Client]
    VB --> Ast[Assets]
    VB --> URLMap[URL to Path Map]
```

**VersionBuilder** contains:
- `main_class`: MainClass
- `java_version`: JavaVersion
- `arguments`: Arguments
- `libraries`: Vec\<Library\>
- `mods`: Vec\<Mod\>
- `natives`: Option\<Vec\<Native\>\>
- `client`: Option\<Client\>
- `assets`: Vec\<Asset\>
- `url_to_path_map`: HashMap\<String, String\>

**Methods**:
- `build_url_map()`: Build complete URL mapping
- `add_url_mapping()`: Add single URL entry
- `remove_url_mapping()`: Remove URL entry

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
