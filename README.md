# PortPeek

A small always-on-top Windows widget that shows which process is holding each
localhost port — and frees it in two clicks.

Built because reclaiming port 8080 usually means `netstat -ano`, squinting at a
PID, then `taskkill`, and discovering the port is *still* held because the thing
listening was a child of the thing you killed.

---

## Install

Download the latest installer from the [Releases](../../releases) page:

| File | Notes |
|---|---|
| `PortPeek_x.y.z_x64-setup.exe` | NSIS installer — **recommended** |
| `PortPeek_x.y.z_x64_en-US.msi` | MSI, for policy-managed installs |

Windows 10/11, **x64 only**. Requires the WebView2 runtime, which ships with
Windows 11; the NSIS installer downloads it automatically if missing.

> **SmartScreen will warn you.** The installer is not code-signed, so Windows
> shows *"Windows protected your PC"*. Click **More info → Run anyway**. Signing
> requires a paid certificate; see [Known limitations](#known-limitations).

## Using it

The widget floats above other windows. Drag it by the header.

| Control | |
|---|---|
| `⟳` | Refresh now (it already polls every 2s) |
| `⏻` | Start with Windows — amber when enabled |
| `⌃` | Collapse to a slim pill showing pinned-port status dots |
| `−` | Hide to the tray (left-click the tray icon to bring it back) |
| `✕` | Quit |

**Freeing a port.** Hover a row, click its `✕`, confirm. Or pin the port as a
favourite and use the chip.

**Favourites** are the strip under the header. `+` to pin a port, right-click a
chip to unpin. Each shows green (free) or amber (occupied, with the process
name), so you can see at a glance whether 8080 is clear without reading the list.

**Filter** matches port, process name, command line, and project label.

## What makes it useful

Two `node.exe` rows are indistinguishable. PortPeek reads each process's command
line and extracts the project that owns it:

```
:8080  node.exe — vite      Acme Ledger · node "C:\...\Acme Ledger\node_modules\vite\bin\vite.js"
:3000  node.exe — next      storefront  · node "C:\...\storefront\node_modules\next\dist\bin\next"
```

The trick is cutting the path at `node_modules`: everything after it is the
runner's own install path and is identical on every project, so the segment
*before* it is the part that identifies anything.

## Killing things safely

Windows makes this less obvious than it looks, so PortPeek does three things:

**Tree-kill by default when it matters.** A dev server started with `npm run dev`
is usually a *child* of a launcher, so terminating the listening PID alone can
leave the server running. The confirm dialog pre-checks "also terminate child
processes" when the parent looks like a launcher (`cmd.exe`, `npm`, `pnpm`,
`yarn`, `bun`…) or the process has children — and leaves it unchecked otherwise.

**Verification by outcome, not exit code.** `taskkill` reports success for kills
that leave the port held. PortPeek ignores the exit code and checks the port on
the next scan:

- port gone → *"Port 8080 freed"*
- port held by a **new** PID → *"respawned by a watcher"* + what to try next
- port held by the **same** PID → reported as a failure

**Shields.** PID 0 and PID 4 have no kill button and are refused in the backend
even if something tries to call it directly. PID 4 rows are badged
`HTTP.sys / Windows` (find the real owner with `netsh http show servicestate`).
Critical processes (`lsass`, `services`, `csrss`, `svchost`…) keep a kill button
but the dialog states plainly that terminating them can bugcheck Windows.

Docker and WSL relays (`com.docker.backend.exe`, `wslrelay.exe`,
`wslservice.exe`, `wslhost.exe`) are labelled and have **no** kill button — the
relay isn't the server, and killing it can break other forwards.

## Known limitations

- **Not code-signed.** SmartScreen warns on first run. A certificate costs money
  and is the only real fix.
- **Elevated processes show limited detail.** Windows returns an empty command
  line for processes owned by another user or running elevated. Those rows fall
  back to name + path, and the header says how many are affected. Run PortPeek as
  administrator for full visibility.
- **Elevated processes can't be killed** unless PortPeek itself runs elevated.
  It never auto-elevates.
- **Docker/WSL detection is by process name**, which varies with Docker Desktop
  version and WSL networking mode (NAT vs mirrored) — see
  [docs/plans/NEEDS-YOU.md](docs/plans/NEEDS-YOU.md).
- **Watcher respawns are detected, not solved.** If a task runner outside the
  killed tree restarts the server, PortPeek tells you rather than fighting it.
- TCP LISTEN only — no UDP, no established connections, no remote listeners.
- Windows only. No macOS/Linux build.

## Configuration

`%APPDATA%\portpeek\config.json` — safe to hand-edit; values are re-validated on
load, and a corrupt file degrades to defaults rather than failing to start.

```json
{ "favorites": [8080], "collapsed": false, "pollIntervalMs": 2000 }
```

Window position and size are stored separately by the window-state plugin.

## Build from source

Needs [Rust](https://rustup.rs) (MSVC toolchain), Node 20+, and the VS Build
Tools C++ workload.

```bash
npm install
npm run tauri dev     # run it
npm run tauri build   # produce installers in src-tauri/target/release/bundle
```

Tests:

```bash
cargo test --lib --manifest-path src-tauri/Cargo.toml
npm run test:labels
```

## How it works

| Layer | |
|---|---|
| Shell | Tauri v2 — frameless, always-on-top |
| Sockets | `netstat2` → `GetExtendedTcpTable`/`GetExtendedTcp6Table` |
| Processes | `sysinfo` (`system` feature only) — name, exe, command line from the PEB |
| Termination | `taskkill /PID <pid> [/T] /F` under `CREATE_NO_WINDOW` |
| Frontend | React + Vite + Tailwind |

No WMI anywhere — `Win32_Process` polling can stall for seconds on a busy
machine, and `sysinfo`'s default features are disabled specifically to keep the
`wmi` crate out of the dependency graph.

The 2s poll runs on a Rust thread rather than a JS `setInterval`, because
WebView2 throttles timers in occluded windows and stretched the poll past its
freshness budget. It skips entirely while the widget is hidden in the tray.
