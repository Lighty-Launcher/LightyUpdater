# Architecture du systeme de scan

## Vue d'ensemble

Le systeme de scan est concu comme une architecture modulaire avec des composants specialises pour chaque type de fichier. L'accent est mis sur la parallelisation et les performances.

## Diagramme d'architecture

```mermaid
graph TB
    subgraph "Entry Point"
        SS[ServerScanner]
    end

    subgraph "Specialized Scanners"
        CS[ClientScanner]
        LS[LibraryScanner]
        MS[ModScanner]
        NS[NativeScanner]
        AS[AssetScanner]
    end

    subgraph "Generic Scanners"
        JS[JarScanner]
        FP[scan_files_parallel]
    end

    subgraph "Utilities"
        Hash[SHA1 Computer]
        Path[Path Normalizer]
        Maven[Maven Converter]
    end

    subgraph "External Services"
        Storage[Storage Backend]
        Models[VersionBuilder]
    end

    SS --> CS
    SS --> LS
    SS --> MS
    SS --> NS
    SS --> AS

    LS --> JS
    MS --> JS
    NS --> FP
    AS --> FP

    JS --> Hash
    FP --> Hash
    JS --> Path
    FP --> Path
    LS --> Maven

    CS --> Storage
    LS --> Storage
    MS --> Storage
    NS --> Storage
    AS --> Storage

    SS --> Models
```

## Composants principaux

### ServerScanner

Point d'entree principal qui orchestre le scan complet d'un serveur.

**Responsabilites**:
- Validation de la structure du serveur
- Coordination des scanners specialises
- Construction du VersionBuilder
- Generation de la URL map

**Methodes**:
- `scan_server`: Scan avec logging detaille
- `scan_server_silent`: Scan sans logging (pour rescans frequents)
- `validate_server_path`: Verification de l'existence du serveur
- `build_version_metadata`: Construction des metadonnees

### JarScanner

Scanner generique reutilisable pour fichiers JAR avec parallelisation.

**Caracteristiques**:
- Generic sur le type de retour `T`
- Fonction de mapping personnalisable
- Controle de concurrence via Semaphore
- Utilisation de futures::stream pour parallelisation

**Structure**:
```rust
pub struct JarScanner {
    pub base_dir: PathBuf,
    pub server: String,
    pub storage: Arc<dyn StorageBackend>,
    pub batch_size: usize,
}
```

**Algorithme**:
1. Collection synchrone des chemins de fichiers JAR
2. Creation du semaphore pour controle de concurrence
3. Stream des fichiers avec `buffer_unordered`
4. Calcul parallele des hashes
5. Mapping des resultats via fonction fournie
6. Filtrage des erreurs et collection

### scan_files_parallel

Fonction utilitaire pour scan parallele de fichiers avec filtre personnalisable.

**Parametres generiques**:
- `T`: Type de retour
- `Filter`: Fonction de filtrage des fichiers
- `Mapper`: Fonction de transformation FileInfo → T

**Utilisation**: Natives et autres fichiers non-JAR necessitant un traitement parallele.

## Scanners specialises

### ClientScanner

Scan du fichier client JAR unique.

**Algorithme**:
```mermaid
flowchart TD
    Start[Start scan_client] --> CheckDir{client/ exists?}
    CheckDir -->|No| ReturnNone[Return None]
    CheckDir -->|Yes| ReadDir[Read directory]

    ReadDir --> FindJar{Find .jar file?}
    FindJar -->|No| ReturnNone
    FindJar -->|Yes| GetName[Extract filename]

    GetName --> ComputeHash[Compute SHA1 & size]
    ComputeHash --> GenURL[Generate URL from storage]
    GenURL --> BuildClient[Build Client struct]
    BuildClient --> ReturnSome[Return Some Client]

    ReturnNone --> End[End]
    ReturnSome --> End
```

**Particularites**:
- Pas de parallelisation (1 seul fichier)
- Prend le premier .jar trouve
- Retourne Option<Client>

### LibraryScanner

Scan des libraries avec conversion en notation Maven.

**Processus**:
```mermaid
sequenceDiagram
    participant LS as LibraryScanner
    participant JS as JarScanner
    participant Maven as path_to_maven_name
    participant Storage

    LS->>JS: new(libraries_dir, server, storage, batch_size)
    LS->>JS: scan(mapper, buffer_size)

    JS->>JS: Collect all .jar paths
    loop For each jar in parallel
        JS->>JS: Compute SHA1
        JS->>Maven: Convert path to Maven name
        Maven-->>JS: "com.example:library:1.0.0"
        JS->>Storage: get_url(remote_key)
        Storage-->>JS: URL
        JS->>JS: Build Library struct
    end

    JS-->>LS: Vec<Library>
```

**Conversion Maven**:
- Input: `com/example/library/1.0.0/library-1.0.0.jar`
- Output: `com.example:library:1.0.0`

### ModScanner

Scan des mods avec structure simple.

**Caracteristiques**:
- Utilise JarScanner comme base
- Pas de conversion de nom (garde le filename)
- Structure plate (pas de sous-dossiers)

**Mapping**:
```rust
Mod {
    name: info.file_name,  // "optifine.jar"
    url: Some(info.url),
    path: Some(info.url_path),
    sha1: Some(info.sha1),
    size: Some(info.size),
}
```

### NativeScanner

Scan des natives avec organisation multi-OS.

**Organisation**:
```
natives/
├── windows/
│   └── lwjgl-natives-windows.jar
├── linux/
│   └── lwjgl-natives-linux.jar
└── macos/
    └── lwjgl-natives-macos.jar
```

**Algorithme**:
```mermaid
flowchart TD
    Start[Start scan_natives] --> CheckDir{natives/ exists?}
    CheckDir -->|No| ReturnEmpty[Return empty Vec]
    CheckDir -->|Yes| InitVec[Initialize all_natives Vec]

    InitVec --> IterOS[Iterate OS types]
    IterOS --> Windows[Scan windows/]
    IterOS --> Linux[Scan linux/]
    IterOS --> MacOS[Scan macos/]

    Windows --> ScanParallel1[scan_files_parallel]
    Linux --> ScanParallel2[scan_files_parallel]
    MacOS --> ScanParallel3[scan_files_parallel]

    ScanParallel1 --> Mapper1[Map with OS tag]
    ScanParallel2 --> Mapper2[Map with OS tag]
    ScanParallel3 --> Mapper3[Map with OS tag]

    Mapper1 --> Extend1[Extend all_natives]
    Mapper2 --> Extend2[Extend all_natives]
    Mapper3 --> Extend3[Extend all_natives]

    Extend1 --> Return[Return all_natives]
    Extend2 --> Return
    Extend3 --> Return
    ReturnEmpty --> End[End]
    Return --> End
```

**Format du nom**:
```rust
name: format!("natives:{}:{}", os, file_name)
// Exemple: "natives:windows:lwjgl-natives-windows.jar"
```

### AssetScanner

Scan recursif de tous les assets.

**Particularites**:
- Scan recursif de toute l'arborescence
- Peut generer des milliers de fichiers
- Utilise walkdir pour traverser les dossiers
- Parallelisation massive avec semaphore

**Structure typique**:
```
assets/
├── minecraft/
│   ├── textures/
│   │   ├── block/
│   │   │   ├── stone.png
│   │   │   └── dirt.png
│   │   └── item/
│   │       └── diamond.png
│   └── sounds/
│       └── ambient/
│           └── cave.ogg
└── custom/
    └── logo.png
```

## Parallelisation

### Architecture de concurrence

```mermaid
graph TB
    subgraph "Main Thread"
        Collect[Collect file paths<br/>sync]
    end

    subgraph "Semaphore Pool"
        Sem[Semaphore<br/>max = batch_size]
    end

    subgraph "Parallel Tasks"
        T1[Task 1: Hash file 1]
        T2[Task 2: Hash file 2]
        T3[Task 3: Hash file 3]
        Tn[Task N: Hash file N]
    end

    subgraph "Result Collection"
        Stream[futures::stream]
        Buffer[buffer_unordered]
        Collect2[Collect results]
    end

    Collect --> Sem
    Sem --> T1
    Sem --> T2
    Sem --> T3
    Sem --> Tn

    T1 --> Stream
    T2 --> Stream
    T3 --> Stream
    Tn --> Stream

    Stream --> Buffer
    Buffer --> Collect2
```

### Controle de concurrence

**Semaphore**:
- Limite le nombre de taches concurrentes
- Evite la surcharge CPU/memoire
- Configuration par batch_size

**buffer_unordered**:
- Execute jusqu'a N futures simultanement
- Collecte les resultats dans l'ordre de completion
- Maximise le throughput

### Exemple de flux

```mermaid
sequenceDiagram
    participant Main
    participant Sem as Semaphore(100)
    participant Task1
    participant Task2
    participant Task100
    participant Task101

    Main->>Sem: Create with capacity 100

    par Task 1-100 start immediately
        Main->>Task1: Spawn
        Task1->>Sem: Acquire permit
        Sem-->>Task1: Permit 1
        Main->>Task2: Spawn
        Task2->>Sem: Acquire permit
        Sem-->>Task2: Permit 2
        Main->>Task100: Spawn
        Task100->>Sem: Acquire permit
        Sem-->>Task100: Permit 100
    end

    Main->>Task101: Spawn
    Task101->>Sem: Acquire permit (wait)

    Task1->>Task1: Compute hash
    Task1->>Sem: Release permit
    Sem-->>Task101: Permit 1 (reused)

    Task101->>Task101: Compute hash
```

## Calcul de hash asynchrone

### Probleme

Le calcul de SHA1 est CPU-intensif et pourrait bloquer le runtime async.

### Solution

Utilisation de tokio pour calcul asynchrone avec buffer:

```mermaid
flowchart TD
    Start[File path] --> OpenFile[tokio::fs::File::open]
    OpenFile --> CreateBuf[Create buffer buffer_size]
    CreateBuf --> LoopRead{Read chunk}

    LoopRead -->|Has data| UpdateHash[Update SHA1 hasher]
    UpdateHash --> LoopRead
    LoopRead -->|EOF| Finalize[Finalize hash]

    Finalize --> Return[Return SHA1 hex + size]
```

**Avantages**:
- Pas de blocage du runtime
- Buffer configurable pour optimiser I/O
- Calcul de size en meme temps

**Configuration**:
```toml
[cache]
checksum_buffer_size = 8192  # 8KB buffer
```

## Integration avec Storage

### Generation d'URLs

```mermaid
sequenceDiagram
    participant Scanner
    participant Storage as StorageBackend
    participant Config

    Scanner->>Scanner: Compute remote_key
    Note over Scanner: format!("{}/{}", server, url_path)

    Scanner->>Storage: get_url(remote_key)

    alt Local storage
        Storage->>Config: Get base_url
        Config-->>Storage: "http://localhost:8080"
        Storage->>Storage: Concatenate base_url + key
        Storage-->>Scanner: "http://localhost:8080/files/survival/mods/optifine.jar"
    else S3 storage
        Storage->>Config: Get public_url
        Config-->>Storage: "https://cdn.example.com"
        Storage->>Storage: Concatenate public_url + bucket_prefix + key
        Storage-->>Scanner: "https://cdn.example.com/servers/survival/mods/optifine.jar"
    end
```

### Remote key format

```
{server_name}/{category}/{relative_path}

Exemples:
- survival/client.jar
- survival/libraries/com/example/lib/1.0.0/lib-1.0.0.jar
- survival/mods/optifine.jar
- survival/natives/windows/lwjgl-natives-windows.jar
- survival/assets/minecraft/textures/block/stone.png
```

## Gestion des erreurs

### Strategie de filtrage

Les erreurs individuelles ne bloquent pas le scan complet:

```rust
let results: Vec<Result<T>> = stream::iter(paths)
    .map(|path| async { /* scan */ })
    .buffer_unordered(batch_size)
    .collect()
    .await;

// Filter errors
Ok(results.into_iter().filter_map(|r| r.ok()).collect())
```

**Impact**: Un fichier corrompu n'empeche pas le scan des autres fichiers.

### Propagation d'erreurs critiques

Certaines erreurs sont critiques:
- Serveur folder inexistant
- Permissions insuffisantes
- Storage backend inaccessible

Ces erreurs sont propagees via `Result<VersionBuilder>`.

## Optimisations

### Collection eagere des paths

```rust
// Bon: Collection sync puis traitement async
let paths: Vec<PathBuf> = WalkDir::new(&dir)
    .into_iter()
    .filter_map(|e| e.ok())
    .filter(|e| is_jar_file(e.path()))
    .map(|e| e.path().to_path_buf())
    .collect();

let results = stream::iter(paths)
    .map(|path| async { /* process */ })
    .buffer_unordered(batch_size)
    .collect()
    .await;
```

**Avantage**: Separe les operations I/O sync (walkdir) des operations async (hash).

### Reuse de Arc

```rust
let storage = Arc::clone(&self.storage);
let mapper = Arc::new(mapper);

// Clone leger pour chaque task
let storage = Arc::clone(&storage);
let mapper = Arc::clone(&mapper);
```

**Avantage**: Pas de copie des structures volumineuses.

### Normalisation de chemins

Conversion des chemins Windows en format Unix:
```rust
// Windows: libraries\com\example\lib.jar
// Unix:    libraries/com/example/lib.jar
normalize_path(path)  // Toujours "/"
```

**Importance**: URLs consistantes independamment de l'OS.
