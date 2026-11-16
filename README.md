# Gyroid

**A modern web-based slicer for 3D printers, with direct Moonraker integration.**

One binary. No Python, no plugins, no sidecars. `gyroid` serves its own UI, slices
meshes to G-code, and drives the printer through [Moonraker](https://moonraker.readthedocs.io/)
over REST and WebSocket.

> **Status: pre-alpha.** This repository is a scaffold. The crate is published at
> `0.0.1` only to reserve the name on crates.io — there is no slicer yet.

## Why

Slicing and machine control are usually split across a desktop app and a browser tab.
Gyroid puts both behind one address: slice a mesh, review the toolpaths, upload it, and
watch the job — without leaving the page or copying files around.

## Planned

- Web UI embedded in the binary — no asset directory, no CDN, works fully offline
- STL / OBJ / 3MF input, G-code out
- Direct Moonraker control: upload, start, pause, live progress over WebSocket
- Single static binary for Linux, macOS and Windows
- Explicit confirmation for every machine-affecting action — never auto-starts a print

## Install

Not yet available. Once released:

```bash
cargo install gyroid
# or grab a binary from the releases page
```

## Development

Requirements: Rust stable, and a Moonraker instance to talk to.

```bash
cargo run -- serve --moonraker http://localhost:7125
cargo check
cargo clippy --all-targets -- -D warnings
```

Read [`AGENTS.md`](AGENTS.md) before contributing — it holds the architecture, the
conventions, the Moonraker safety rules, and the project's current status.

## License

MIT. See [`LICENSE`](LICENSE).
