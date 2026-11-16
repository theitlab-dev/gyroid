# AGENTS.md — Gyroid

> **Read this file first, before touching anything.** Then update it if your work
> changed any fact recorded here (see [Maintaining this file](#maintaining-this-file)).

Basic overview of the Gyroid project, its architecture, and the rules for working in it.

---

## 1. What this is

**Gyroid** is a modern, web-based slicer for FFF/FDM 3D printers that also controls the
machine directly through the **Moonraker** API.

It is a **single statically-linked Rust binary**. One file you download and run. It:

- serves its own web UI (embedded in the binary — no asset directory, no CDN),
- slices meshes (STL / OBJ / 3MF) into G-code,
- talks to Moonraker over REST + WebSocket to upload jobs, watch progress, and control
  the printer.

No Python, no Klipper plugin, no sidecar containers. `gyroid` + a browser is the whole
install.

| | |
|---|---|
| Tagline | *Slice and drive Klipper from one binary.* |
| Status | **Pre-scaffold.** Design decided, no code written yet. |
| Language | Rust (edition 2024) |
| License | **TBD — ask the maintainer before assuming** |
| Repo root | `/home/darthrax/gyroid` |
| Maintainer | TBD — see [Open questions](#9-open-questions) |

### Non-goals (for now)

- Not a Moonraker replacement, not a Klipper host image.
- Not a general CAD tool. Meshes in, G-code out.
- No printer firmware flashing, no macro editing (read-only macro list is fine).
- No cloud account, no telemetry, no phone-home.

---

## 2. Locked naming & distribution decisions

Verified 2026-11-16. **Do not rename these without asking the maintainer.**

| Thing | Value | Status |
|---|---|---|
| Product name | **Gyroid** | decided |
| crates.io crate | `gyroid` | free as of 2026-11-16 — **reserve it** |
| Binary name | `gyroid` | — |
| npm package (UI) | `gyroid` | free (only if we ever need it) |
| GitHub org handle | `gyroid` | free (user handle taken; org is not) |
| Canonical domain | `https://gyroid.theitlab.dev` | maintainer owns `theitlab.dev` (wildcard DNS, Cloudflare) |
| Landing/docs | `gyroid.theitlab.dev` | same host, `/` and `/docs` |

Rationale, for the record: "kerf" was the original favourite and was rejected — the
crates.io name is a stale tokio trace collector, `trykerf.com` is a live laser-CAM
product, and 541 GitHub repos already use the name. `gyroid` is free on crates.io *and*
npm, has no competing software product (only infill explainer articles), and is the
signature 3D-printing surface. Known weakness: it is a generic technical term, so it is
a weak trademark.

---

## 3. Current state

Nothing is implemented yet. Written 2026-11-16:

- [x] Name chosen and availability-verified (crates.io, npm, GitHub, domains)
- [x] Architecture and crate choices decided (below)
- [x] This file
- [ ] Everything else

### Confirmed environment (the dev box)

This repo currently lives **on the printer host** — a Raspberry Pi 4.

| | |
|---|---|
| Arch / OS | `aarch64` / Debian 12 (bookworm) |
| Hardware | 4 cores, 7.5 GB RAM, 46 GB free |
| Moonraker | **live** at `http://localhost:7125` — `GET /server/info` returns 200 |
| Klipper / Mainsail | installed (`~/klipper`, `~/mainsail`, `~/printer_data`) |
| Node / npm | v22.23.4 / 10.9.9 |
| **Rust** | **NOT installed** — no `cargo`, no `rustup` |

Consequences: release builds here will be slow (4 cores, SD card). Prefer
`cargo build`/`cargo check` during development, and build distribution binaries in CI or
on a faster machine. If you need a faster linker, prefer `lld`; do not add `mold` on
this box without asking.

---

## 4. Architecture

Rule of thumb: **the binary must be shippable as one file on Linux, macOS and Windows.**
Anything that breaks that (a required external asset, a runtime dependency, a sidecar
service) is a design error, not a trade-off.

```
                 ┌─────────────────────────────────────────┐
   browser ──────┤  axum: static UI (embedded) + REST/WS    │
                 │                                          │
                 │  slicer core ── geometry ── mesh I/O     │
                 │      │                                   │
                 │      └── gcode emitter                   │
                 │                                          │
                 │  moonraker client ── REST + WebSocket    │
                 │  job queue / history ── sqlite           │
                 └───────────────┬─────────────────────────┘
                                 │ http/ws :7125
                             Moonraker ── Klipper
```

### Planned crates

| Concern | Choice | Notes |
|---|---|---|
| HTTP server | `axum` + `tokio` | serve UI + API on one port |
| Static UI | `rust-embed` | **must** embed; no external `dist/` at runtime |
| Moonraker WS | `tokio-tungstenite` | `/websocket` for live status |
| Moonraker REST | `reqwest` | upload, start, query |
| Math | `nalgebra` | transforms, vectors |
| 2D booleans / offsetting | `i_overlay`, `geo` | thin area in Rust — see below |
| 3D collision | `parry3d` | support checks, later |
| Mesh IO | `stl_io`, `obj-rs`, 3MF via `quick-xml`/`zip` | 3MF is a zip of XML |
| Persistence | `rusqlite` (bundled) | jobs, history, settings |
| Errors | `thiserror` (lib), `anyhow` (bin) | no `unwrap()` in library code |
| Logging | `tracing` + `tracing-subscriber` | structured, stderr |
| CLI | `clap` | `gyroid serve`, `gyroid slice`, etc. |

**Known hard truth:** there is no mature Rust slicing or CAD kernel to build on.
`cleave` (crates.io, "from-scratch geometry kernel for CAD") is abandoned at 28
downloads. Expect to write polygon offsetting, infill generation, and layer planning
ourselves. Scope the first milestone accordingly — it is not a weekend job.

---

## 5. Commands

Rust is not installed here. Do **not** install a toolchain or run `curl | sh` without
asking the maintainer first — record the decision in [Open questions](#9-open-questions).

Once scaffolded, these are the expected commands:

```bash
cargo run -- serve --moonraker http://localhost:7125   # dev server, open :8080
cargo run -- slice model.stl -o model.gcode            # headless slice
cargo check                                            # fast feedback
cargo clippy --all-targets -- -D warnings              # must be clean
cargo fmt --all                                        # always before commit
cargo test                                             # unit tests
cargo build --release                                  # slow on this box
```

UI (only if a JS build step is introduced — currently undecided):

```bash
npm --prefix ui install
npm --prefix ui run build    # output must be embedded into the binary
```

---

## 6. Moonraker integration

Documented behaviour as of 2026-11-16; verify against the live instance before relying
on it.

- Base URL default `http://localhost:7125`, overridable via CLI flag or config.
- `GET /server/info` — reachability + version probe.
- `GET /printer/info`, `GET /printer/objects/query?...` — state and object queries.
- `POST /printer/gcode/script` — run a macro/gcode line.
- `POST /server/files/upload` — multipart upload of G-code.
- `POST /printer/print/start?filename=...` — begin a job.
- `GET /server/files/list` — job history.
- `WS /websocket` — subscribe to `printer.objects.subscribe` for live progress.
- Restart Klipper only via `POST /printer/restart` **after explicit user confirmation**.

### Safety rules (non-negotiable)

1. **Never** auto-start a print, heartbeat, or job. A human confirms every one.
2. Never send E-steps, PID, or endstop mutations without an explicit confirm dialog.
3. Never write to `printer_data/config/` — that is Moonraker's/ Mainsail's territory.
4. Any G-code we emit must be reviewable in the UI before upload.
5. Destructive actions (restart, emergency stop) require a distinct, non-default control.
6. Fail loudly and read-only when Moonraker is unreachable — never silently retry a print.

---

## 7. Conventions

- `rustfmt` defaults, `clippy -D warnings` in CI.
- Library modules return typed errors (`thiserror`); `anyhow` only at the binary edge.
- No `unwrap()`/`expect()` in library code. In tests it is fine.
- Public items get doc comments. Modules get a one-line `//!` header.
- Feature work lands as small commits; **conventional commits** (`feat:`, `fix:`,
  `docs:`, `refactor:`, `chore:`).
- Any new dependency needs a one-line justification in the PR/commit body — especially
  anything that pulls in C, or anything that breaks the single-binary goal.
- Do not commit secrets. Tokens live in `~/.cargo/credentials.toml` (never in this repo)
  and printer credentials in local config, not source.

---

## 8. Maintaining this file

This file is **not** write-once. It is the project's memory, and keeping it true is part
of every task — treat staleness here as a bug.

### Update it when any of these happen

| Trigger | Which section |
|---|---|
| A crate is added, removed or replaced | §4 table |
| A build/run/test command changes | §5 |
| Rust toolchain / OS / hardware changes on the dev box | §3 |
| Moonraker endpoint or behaviour is discovered to differ | §6 |
| A design decision is made or reversed | §2, §9, and the log |
| The project moves from pre-scaffold to working | §1 status, §3 checklist |
| License, maintainer or author identity is decided | §1, §9 |
| A convention is added or dropped | §7 |
| Repo layout changes materially | §4 |

### Rules for editing this file

1. **Edit in place, keep it short.** This is an overview, not documentation. If a topic
   outgrows its section, move it to `docs/` and leave a one-line pointer here.
2. **Never delete the rationale.** If a decision is reversed, keep the old reasoning in
   the entry and say why it changed. Future agents repeat mistakes without it.
3. **Mark uncertainty explicitly** — "TBD", "unverified", "as of <date>". Never state a
   guess as fact.
4. **Verify, don't assume.** Commands, endpoints, and availability claims should be
   checked against reality before being recorded.
5. **Date every factual claim that can rot** (tool versions, availability checks,
   observed API behaviour).
6. **Append to the update log** below with a one-line summary. Do not rewrite history.
7. If you are unsure whether a change belongs here, add it to
   [Open questions](#9-open-questions) instead of guessing.

### Update log

| Date | Change |
|---|---|
| 2026-11-16 | File created. Name locked to `gyroid` after availability sweep (crates.io, npm, GitHub, domains). Architecture and crate choices recorded. Dev box confirmed as Klipper/RPi host with no Rust toolchain. |

---

## 9. Open questions

Answer these before or during scaffolding, and record the answers above.

1. **License?** MIT / Apache-2.0 / dual / AGPL. Affects whether this can be packaged.
2. **Maintainer identity** for `Cargo.toml` `authors`, git commits, and the crates.io
   publisher — needs a name and email.
3. **Install the Rust toolchain on this Pi, or develop elsewhere?** (aarch64, 4 cores —
   workable but slow.) Nothing is installed yet.
4. **UI approach:** hand-written HTML/JS embedded directly, or a JS build step
   (Vite/React/Svelte) whose output is embedded? Affects §5 and the "single binary" rule.
5. **Slicing license strategy:** writing the kernel from scratch vs. binding to
   `libslic3r`/`OrcaSlicer` via C++ FFI (fast, but complicates single-binary builds and
   is AGPL). This is the biggest technical decision outstanding.
6. **Git:** initialise a repo now? (No git identity is configured on this machine yet.)
7. **Reserve `gyroid` on crates.io now** as a stub, before someone takes it?
