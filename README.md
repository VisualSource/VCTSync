# VCTSync

Tools for modding [Void Crew](https://store.steampowered.com/app/1063420/Void_Crew/) and for developing the
[VoidCrewTerminus](https://github.com/VisualSource/VoidCrewTerminus) mod:

- **vcl** — a desktop mod manager and launcher. Isolated BepInEx profiles, a Thunderstore catalog, installs
  from `.zip`, VoidCrewTerminus builds (GitHub releases and dev builds), and a BepInEx log viewer.
- **vct-agent** — a small HTTP service for the Linux dev box. It serves fresh VoidCrewTerminus build zips to
  vcl and passes files between the dev box and the Windows machine.

vcl's UI is written in React/TypeScript and rendered as native [iced](https://iced.rs) widgets by
**iced-js**, a crate in this repo that runs the UI in an embedded JavaScript engine (QuickJS).

## Repository layout

| Path | What it is |
|---|---|
| `apps/vcl` | The launcher. Rust host in `src/`, React UI in `view/`, HTML design mockups in `mockups/`. |
| `apps/vct-agent` | The dev-box agent (axum HTTP server + CLI). |
| `apps/iced-js-example-app` | Minimal app showing how to host an iced-js UI. |
| `crates/iced-js` | React → iced renderer: QuickJS runtime, JSX elements, host ⇄ JS IPC. |
| `crates/bepinex-lib` | BepInEx profiles: install the BepInExPack and mods, enable/disable, uninstall, launch via Doorstop. |
| `crates/iced-query`, `crates/iced-xml` | Experiments (async queries for iced, XML-style UI macro). Not used by the apps. |
| `docs/` | Design notes: [iced-js architecture](docs/iced-js-architecture.md), [how BepInEx is launched from a profile](docs/bepinex-launch-research.md). |

## Prerequisites

- **Rust** (stable, edition 2024 — 1.85 or newer).
- **Node.js** and **pnpm**. The build scripts call `pnpm` to bundle the JavaScript, so it must be on `PATH`.
- **Windows** to launch the game from vcl. Everything else builds on Linux and macOS too, and vct-agent is meant
  for Linux.

## Building and running vcl

```sh
# once: install the UI's JavaScript dependencies
cd apps/vcl && pnpm install && cd ../..

cargo run            # vcl is the workspace's default member
```

`cargo build` also bundles the UI: `apps/vcl/build.rs` runs Vite over `view/`, and iced-js's build script
bundles its React runtime (running `pnpm install` for it on first build). Both are embedded into the binary.

Debug builds read the UI bundle from `apps/vcl/dist` at runtime (release builds embed it), so UI changes don't
need a Rust rebuild: run `pnpm exec vite build --watch` in `apps/vcl` and press **F5** in the app to reload the
UI. Changes to the Rust host need `cargo run` again.

### Where vcl keeps its data

Everything lives in one folder in the OS config directory — on Windows
`%APPDATA%\vcl.visualsource.github\`:

- `settings.ron` — app settings (theme, Steam folder, launch method, dev agent).
- `profiles.ron` — the profile list and each profile's mods.
- `profiles\<id>\` — each new profile's BepInEx install (`BepInEx\plugins`, `config`, `LogOutput.log`, …).

Launching a profile copies BepInEx's loader files (`winhttp.dll`, `doorstop_config.ini`) into the game folder
and points them at the profile. **Settings → Game → Restore vanilla game** removes them again.

## vct-agent

Runs on the machine that builds VoidCrewTerminus and makes its build zips installable from vcl's
**Versions → Dev builds** tab.

```sh
cargo build --release -p vct-agent

# write config.ron next to the binary (prints the generated token)
./target/release/vct-agent init

# edit config.ron: point scan_dirs at the folders your build writes VoidCrewTerminus-*.zip to
./target/release/vct-agent run
```

Then in vcl, **Settings → Dev agent**: set the address to the agent's `host:port` (default port `9787`) and
paste the token. **Test** confirms the connection.

Other commands: `vct-agent status` (config summary and whether an agent is running), and
`vct-agent send <file>` (stage a file in the outbox). `--config <path>` points any command at a different
config file.

vcl currently uses `/versions` only. The file transfer endpoints (`/files`, `/outbox`) and `/events` are there
for clients but nothing in vcl calls them yet.

Every endpoint needs `Authorization: Bearer <token>`:

| Endpoint | |
|---|---|
| `GET /versions` | Local builds, in the same shape vcl uses for GitHub releases. |
| `GET /versions/{id}/download` | The build zip. |
| `GET /status` | Bind address, build count, connected clients, outbox size. |
| `POST /files?name=<file>` | Upload a file into `incoming_dir`. |
| `GET /outbox`, `GET` / `DELETE /outbox/{name}` | Files staged with `vct-agent send`. |
| `GET /events` | Server-sent events: `build`, `file`, `ping`. |

## Development

### Tests

```sh
cargo test -p bepinex-lib
cargo test -p vcl
cargo test -p vct-agent
cargo test -p iced-js --lib

# vcl UI: type-check, and the log parser check
cd apps/vcl
node_modules/.bin/tsc --noEmit -p .
node --experimental-strip-types tests/parse-log.check.ts
```

### Working on the vcl UI

- Pages live in `apps/vcl/view/pages/`. Elements are lower-case JSX tags (`<col>`, `<row>`, `<view>`, `<text>`,
  `<button>`, `<scroll>`, …) typed in `crates/iced-js/package/elements.d.ts`.
- The view calls the Rust host with `invoke(cmd, payload)` from `react-iced-native`. The handlers are the
  `match cmd.as_str()` in `apps/vcl/src/app.rs`. The host can push events to the view; the view subscribes
  with `listen(cmd, callback)`.
- Layout follows iced, not CSS: a `<col>` only aligns children horizontally and a `<row>` only vertically. To
  center both ways, wrap the content in `<view center="fill">` (or use `Center` from `view/utils.tsx`).
- `apps/vcl/mockups/*.html` are the design mockups for each page. Open them in a browser; the **Widget names**
  toggle shows which element each part maps to.
