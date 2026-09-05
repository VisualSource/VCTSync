# vct-sync — design

Replaces the `VoidCrewTerminus/scripts/installer/` tool and the `~/repos/game-sync`
CLI. Manages installs of the VoidCrewTerminus mod (local dev builds + published
GitHub releases), streams/collects the game log, and moves files between the two
machines.

The mod is developed on a **Linux** box and the game runs on a **Windows** box,
so this is a networked app: builds flow Linux→Windows, logs flow Windows→Linux.

## Shape

Cargo **workspace**, three crates:

| Crate | Runs on | Purpose |
|-------|---------|---------|
| `vct-core` | both | wire protocol types, GitHub releases client, build-dir scanner (zip + `manifest.json` introspection), install / `mods.yml` logic (ported from `install.ts`), YAML config load/save |
| `vct-agent` | Linux dev box | headless daemon |
| `vct-sync` | Windows game box | iced GUI, 4 tabs: **Builds / Logs / Transfer / Settings** |

The existing `src/` (partial Versions screen, `Query<D>` abstraction, GitHub fetch)
is restructured into `vct-core` + `vct-sync`; `Query<D>` and the fetch code carry over.

`iced = "0.14"` is pinned and resolves in `Cargo.lock` — staying on it.

## `vct-agent` CLI

| Command | Effect |
|---------|--------|
| `vct-agent init` | write a commented default `config.yml` next to the executable, exit |
| `vct-agent run` | serve; error clearly on missing required fields |
| `vct-agent send <file> [--name]` | stage a file into the outbox, fire an SSE `file` event |
| `vct-agent status` | bind addr, connected GUI count, pending outbox, watched dirs |

The running agent watches the outbox dir, so `send` works whether or not `run` is live.

## Transport

Agent runs a **plaintext HTTP** server on the LAN. Shared **bearer token**:
`init` generates it into the agent config; the GUI config carries the same string;
every request needs `Authorization: Bearer <token>`, else `401`. Bind address
configurable, default `0.0.0.0:9787`. File endpoints reject path traversal and cap
upload size.

| Endpoint | Purpose |
|----------|---------|
| `GET /builds` | local builds: `[{id, version, deps, config, mtime, size, sha256}]` |
| `GET /builds/:id/download` | zip bytes |
| `POST /files` (name, bytes) | write into the Linux **incoming dir** (Windows→Linux: logs, screenshots) |
| `GET /outbox`, `GET /outbox/:name` | staged Linux→Windows files |
| `GET /events` | **SSE**: `build` events (new zip in a scan dir) + `file` events (new outbox item) |

The GUI holds `/events` open while running. If the GUI is closed, Linux→Windows
pushes queue in the outbox until it reconnects.

The GUI still works with the agent unreachable: GitHub releases, local install, and
live-log tail need no network. Only the local-builds list and file transfer need the agent.

## Config

**YAML**, `config.yml` in the **same directory as the running binary** (resolved via
`current_exe()`, not the working directory). `--config <path>` overrides. One format
project-wide (also covers reading/writing TMM's `mods.yml`).

Agent keys: `bind`, `token`, `scan_dirs` (default: the Debug + Release
`net472/Releases/` dirs), `incoming_dir`, `outbox_dir`.
GUI keys: `agent` (address), `token`, `profile` + last-used memory, save dirs.

## Builds tab

- **GitHub**: published releases only, no auth/token ever. Filter `draft == false`,
  label prereleases, pick the `.zip` asset (skip source archives). Manual **Refresh**
  + short cache to respect the 60/hr anonymous limit.
- **Local**: from the agent. Agent watches `scan_dirs`, reads `manifest.json` from
  inside each zip, pushes a `build` SSE event on change. GUI shows a toast with
  **[Install]** — **always a manual click, no auto-install**.
- Header shows the **currently-installed version**, read from the selected profile's
  `mods.yml` entry.

## Install (full `install.ts` parity, ported to Rust)

1. Autodetect `%APPDATA%\Thunderstore Mod Manager\DataFolder\VoidCrew\profiles\`
   (manual path fallback).
2. Pick profile (last-used remembered).
3. Wipe + extract zip to `BepInEx/plugins/VisualSource-VoidCrewTerminus/`.
4. Upsert the `mods.yml` entry: `enabled`, preserve `installedAtTime`,
   manifest-first dependencies.
5. Leave `BepInEx/config` alone.

Source zip: from the agent (local build) or a direct GitHub download (release).

## Logs tab

GUI tails `<profile>/BepInEx/LogOutput.log` **locally** (no network), reopening on
truncation / relaunch (size shrink). **"mod lines only"** toggle filters to lines
containing `VoidCrew`. **Collect** bundles the log (full or filtered, timestamped),
`POST`s it to the agent's incoming dir, and keeps a local copy.

## Transfer tab

- **Windows→Linux**: file picker → `POST /files`. Plus a **recent-screenshots
  quick-pick** from `Pictures\Screenshots` and `Downloads`.
- **Linux→Windows**: `vct-agent send <file>` → SSE `file` event → toast with
  **[Save]** (or auto-save to a configured dir).

## Out of scope

- **Live Unity runtime viewing** — deferred.
- **Static/browsable file server** (`serve.ts` / `fileserver.exe`) — dropped; an
  **SMB mount** covers browsing artifacts.

## Cleanup (only after parity is verified)

- Move `~/repos/game-sync` → `~/repos/archive/game-sync`.
- `git rm` `VoidCrewTerminus/scripts/installer/` with a commit in that repo.
