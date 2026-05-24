# Contributing

Thanks for your interest! Here's the fast path from a fresh clone to a
merged PR.

## Reporting issues

Open one [here](https://github.com/Lighty-Launcher/LightyUpdater/issues)
and tell us:

- The command you ran (`cargo run -p lighty-app`, `cargo run -p lighty -- ...`)
- The full error / panic backtrace
- Your OS + `rustc --version`
- Your `config.toml` (redact credentials)
- What you expected to happen

For security findings, open a private security advisory instead of a
public issue.

## How the repo is laid out

```
.
├── app/                       # `lighty-app` binary (server entry point)
├── cli/                       # `lighty` binary (instance manager)
├── crates/
│   ├── models/                # Domain models (VersionBuilder, Library, Mod, ...)
│   ├── events/                # EventBus + AppEvent enum + CompositeEventSink
│   ├── adapters/              # Config-IO loader, ConsoleEventSink
│   ├── utils/                 # SHA1, path helpers
│   ├── filesystem/            # FileSystem ops + MIME detection
│   ├── config/                # Config schema (pure, no I/O)
│   ├── storage/               # Backend abstraction (local / S3 / R2)
│   ├── scanner/               # Parallel scan of client/libs/mods/natives/assets
│   ├── file-diff/             # FileDiff::compute (granular change detection)
│   ├── cdn/                   # Cloudflare / CDN HTTP clients + CdnEventSink
│   ├── file-cache/            # Moka-backed LRU for served file payloads
│   ├── rescan/                # Rescan loops, file watcher, change detector
│   ├── cache/                 # Facade composing the four crates above
│   ├── watcher/               # Hot-reload for config.toml
│   ├── api/                   # Axum HTTP layer
│   ├── runtime/               # Composition root
│   └── lib.rs                 # `lighty-updater` re-export facade
├── Cargo.toml                 # Workspace manifest
├── LICENSE
└── README.md
```

Versions are declared once in the root `Cargo.toml` via
`[workspace.package]` and inherited by every crate with
`.workspace = true` — **don't bump individual crates**, always the
workspace.

## Branches

- **`development`** — where PRs land.
- **`production`** — release branch, only updated at release time.

Branch off `development`, never push directly to `production`.

## Local setup

```bash
rustup toolchain install stable      # MSRV is 1.95, edition 2024
git clone https://github.com/Lighty-Launcher/LightyUpdater
cd LightyUpdater
git checkout development
cargo check --workspace
```

Run the server:
```bash
cargo run -p lighty-app
# or with S3 / R2 support:
cargo run -p lighty-app --features s3
```

Manage instances with the CLI:
```bash
cargo run -p lighty -- get
cargo run -p lighty -- create survival --dir ./instances/survival
cargo run -p lighty -- start survival
```

Config: `config.toml` in the working directory (auto-created on first
run). Override the path via `LIGHTY_CONFIG=/abs/path/to/config.toml`.

## Heads-up: `.gitignore` is whitelist-style

The repo ignores everything by default and only allows specific paths:

```
/*

!.gitignore
!README.md
!CONTRIBUTING.md
!LICENSE
!Cargo.toml
!Cargo.lock

!.github/**
!.assets/
!app/
!cli/
!crates/
```

If you add a new top-level file or folder, **whitelist it in
`.gitignore`** or git will silently skip it.

## Sending a PR

1. Branch off `development`:
   ```bash
   git checkout development && git pull
   git checkout -b feat/<short-name>     # or fix/, refactor/, docs/
   ```
2. Implement, then run the local checks listed above.
3. Push and open a PR targeting `development`.
4. Wait for review — you may be asked to rebase before merge.

Keep PRs focused. Multi-purpose work is OK if the commit message makes
the scope obvious.

## Code style

- **No fully-qualified paths inline** — add a `use` at the top of the file.
- **Comments**: above the code, short, explain *why* not *what*. Inline
  trailing notes are reserved for trivial one-liners.
- **No single-letter variable names** — not in tests, not in closures,
  not in `Err(_)` bindings. Prefer `error`, `entry`, `server`, `change`,
  etc.
- **Errors via `thiserror`**, one error enum per crate; surface upper
  errors with `#[from]` for transparent propagation.

## Adding a dependency

New crate in `Cargo.toml`? **Justify it** in the commit message (or PR
description): what it brings, why an existing dep or a few lines of
hand-written code wouldn't do. Lighter is always preferred — every
extra dep is build time, supply-chain surface, and one more thing to
keep up to date.

## Versioning (CalVer)

Versions look like `YY.MM.PATCH` (e.g. `26.5.0` = May 2026, patch 0):

| Part | Bump when |
|---|---|
| `YY` | New calendar year |
| `MM` | New monthly release (may include breaking changes) |
| `PATCH` | Bug fix / non-breaking addition inside the same month |

API stability is **per-month**, not per-major-number. Consumers pin to
a `YY.MM` and migrate on their own pace.

## Commits

**Commit messages must be clear.** Six months from now, the title
alone should tell you what changed and why.

Prefer the Conventional Commits style with an optional scope:

```
feat(rescan): debounce file watcher events per server
fix(cdn): retry on transient 5xx from Cloudflare
refactor(cache): extract Moka LRU into lighty-file-cache
docs(architecture): refresh dependency matrix
ci(release): publish crates in dependency order
```

Release commits land on `production` with the format
`release YY.MM.PATCH: <description>`.

## Architecture rules

Each crate ships its own `docs/` folder with `overview.md`,
`how-to-use.md`, `exports.md` and optional flow diagrams. Read the
relevant crate's `overview.md` before changing it — they list the
public surface and the rules of the road for that layer.

Cross-crate side effects flow through `lighty-events` — do not add
direct calls from `lighty-rescan` to `lighty-cdn` (or any equivalent
shortcut) without a corresponding event variant.

## Licence

MIT, declared in `Cargo.toml`. By contributing, you agree your work is
published under the same licence.
