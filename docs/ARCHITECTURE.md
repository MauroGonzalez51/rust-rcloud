# Architecture

rcloud is a CLI wrapper around [rclone](https://rclone.org/). It records
local ↔ remote path mappings, applies a configurable hook pipeline around each
transfer, and skips transfers whose content has not changed. rclone does the
actual network I/O; rcloud manages configuration, orchestration, and
transformation.

## Crate layout

```
src/
├── main.rs            # binary entry point -> cli::run::run
├── lib.rs             # library root and re-exports
├── cli/               # argument parsing, command dispatch, handlers
│   ├── parser.rs      # clap definitions (Cli, Commands, GlobalParameters)
│   ├── run.rs         # top-level dispatch
│   ├── context.rs     # CommandContext<L> shared state
│   ├── macros.rs      # handler-import and context-construction macros
│   └── commands/      # remote / path / sync / configure handlers
├── config/            # persisted state and config types
│   ├── app.rs         # AppConfig (rcloud.toml)
│   ├── registry.rs    # Registry (registry.json)
│   ├── remote.rs      # Remote
│   ├── path_config.rs # PathConfig, PathConfigHooks
│   ├── hook_config.rs # Hook trait, Hooks, HookExecType, register_hooks!
│   └── tags.rs        # tag selection prompt
├── hooks/             # Hook trait impls
│   ├── macros.rs      # register_hooks!, define_hook!
│   ├── hook_context.rs# HookContext, HookContextMetadata
│   ├── hook_builder.rs# HookBuilder, builder traits
│   ├── zip/           # Zstd compression
│   ├── encryption/    # AES-256-GCM
│   └── backup/        # rotated replicas
├── tui/               # ratatui/crossterm interactive UI
└── utils/             # hashing, directories, path expansion, logging
```

## Configuration and state

Two files, both under the platform config directory by default (overridable
with `--config` / `--registry`):

- **`rcloud.toml`** → [`AppConfig`]. Core settings (temp directory) and TUI key
  bindings. If missing, it is created from the embedded
  `assets/default_config.toml` (via `rust-embed`).
- **`registry.json`** → [`Registry`]. The mutable store of `remotes` and
  `paths`.

### Registry safety

Reads take an exclusive advisory file lock (`fs2`). Mutations go through
`Registry::tx`, which snapshots the in-memory state, applies the closure, and
saves; if the save fails, the snapshot is restored so a failed write leaves the
registry unchanged.

A [`Remote`] only references a remote already configured in `rclone` itself (by
name) — rcloud does not manage rclone's own config. A [`PathConfig`] maps a
local path to a remote path and carries: a content `hash` (for skip detection),
`tags` (for batch operations), and `hooks` split into `push` and `pull`
pipelines.

## Request flow

```
main -> cli::run::run
          ├── parse args (clap)
          ├── init logger
          ├── load AppConfig + Registry
          ├── build CommandContext
          └── dispatch:
               ├── subcommand -> handler (remote/path/sync/configure/completion)
               └── none       -> tui::run::run_tui
```

[`CommandContext<L>`] carries the loaded config, global flags, and the shared
registry behind `Arc`s, plus a generic `local` payload `L` for
command-specific arguments. `with_args` attaches typed arguments;
`with_registry` returns a locked guard. The `use_handlers!` and
`command_context!` macros cut boilerplate around importing handlers and building
contexts.

The TUI (`tui/`) presents the same operations as a navigable tree menu and,
when an action is selected, drops out of the alternate screen and calls the very
same handlers the CLI uses.

## Sync engine

Both directions live in `src/cli/commands/sync/utils/`. The unit of work is a
single `PathConfig`; `sync all` simply filters paths by tag and loops
`sync_single` over each.

### Push (`push.rs`)

1. Hash the local content (`utils::Hash::hash_path`).
2. `force()` — unless `--force`, skip when the new hash matches the stored one.
3. Run push hooks in order via `execute_hooks`, threading a `HookContext`.
4. Compute the remote filename (`compute_remote_filename`) and rename the
   processed output if a hook changed the name.
5. Upload with `rclone` (`execute_rclone`: `copyto`/`copy` + `--checksum
   --transfers=8 --checkers=16`).
6. On success, persist the new hash via `Registry::tx`.

### Pull (`pull.rs`)

1. Download the remote content into a temp directory with `rclone`.
2. Run pull hooks in **reverse** order (undoing the push pipeline).
3. Hash the result; `force()` decides whether to skip. `--clean` clears the
   destination first.
4. Move the processed content to the local path and persist the new hash.

### Skip detection

`PathConfig.hash` stores the content hash of the last successful sync. On the
next run the freshly computed hash is compared; a match short-circuits the
transfer. `--force` bypasses this.

## Hook pipeline

Hooks are the extension point. A hook implements the [`Hook`] trait and
transforms `HookContext.path`. The [`register_hooks!`] macro wires each hook's
config/runtime types and metadata (filename impact, descriptions, shared-config
support) into `HookConfig`, and [`define_hook!`] generates the paired
config/runtime structs. Built-in hooks:

| Hook       | Push                       | Pull                    | Notes                                   |
| ---------- | -------------------------- | ----------------------- | --------------------------------------- |
| Zip        | Zstd compress to archive   | Extract                 | Records `ZipChecksum`; changes filename |
| Encryption | AES-256-GCM encrypt        | Decrypt                 | Per-file random salt; changes filename  |
| Backup     | Rotated local/remote copy  | Rotated copy            | Does not change the path                |

See [`HOOKS.md`](HOOKS.md) for how to add one.

### Encryption format

Encrypted files use the layout:

```
[ MARKER (16) | SALT (16) | NONCE (12) | CIPHERTEXT ]
```

The password is verified against a stored Argon2 hash, then the AES key is
derived per file from the random 16-byte salt stored in the header. Because the
salt is random and per file, the same password produces a different key for
every file, and there is no global salt to precompute against.

## External dependency

rcloud shells out to the `rclone` binary (resolved from `--rclone`, the
`RCLONE_PATH` env var, or `rclone` on `PATH`). `configure` checks availability
by invoking `rclone version`. All transfers are delegated to rclone; rcloud
never talks to cloud providers directly.

[`AppConfig`]: ../src/config/app.rs
[`Registry`]: ../src/config/registry.rs
[`Remote`]: ../src/config/remote.rs
[`PathConfig`]: ../src/config/path_config.rs
[`CommandContext<L>`]: ../src/cli/context.rs
[`Hook`]: ../src/config/hook_config.rs
[`register_hooks!`]: ../src/hooks/macros.rs
[`define_hook!`]: ../src/hooks/macros.rs
