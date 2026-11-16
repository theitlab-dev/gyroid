# AGENTS.md — Gyroid

> **Read this file first, before touching anything.** Then update it if your work
> changed any fact recorded here (see [Maintaining this file](#8-maintaining-this-file)).

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
| Status | **Name claimed, v0.0.1 published. No slicer yet** — see §3 |
| Language | Rust, edition 2024, MSRV 1.85 |
| License | **MIT** — chosen 2026-11-16 by the agent as the low-risk default; **maintainer has not confirmed** (§9.2) |
| Repo root | `/home/darthrax/gyroid` |
| Maintainer | `darthrax` — GitHub [@darthrax](https://github.com/darthrax) |
| Org | GitHub `theitlab-dev` |
| Repo | **live:** https://github.com/theitlab-dev/gyroid (public, `main`, SSH remote) |

### Design stance

Written **from scratch**, as a **modern remake of very old software** in this space —
not a port, not a fork, not a wrapper around an existing slicer. The intent is to take
what those tools got right and rebuild it with a modern architecture: a single binary,
a real web UI, and direct printer integration instead of filesystem-and-prayer.

> **TBD:** the specific ancestor/ancestors being remade are not yet recorded. Ask the
> maintainer and fill this in — it materially shapes feature scope. See §9.

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
| crates.io crate | `gyroid` | **CLAIMED** 2026-11-16, `v0.0.1` published; `cargo install gyroid` verified from the registry |
| Binary name | `gyroid` | — |
| npm package (UI) | `gyroid` | free (only if we ever need it) |
| GitHub org handle | `gyroid` | free (user handle taken; org is not) |
| Repo | `https://github.com/theitlab-dev/gyroid` | **created** 2026-11-16, public |
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

As of 2026-11-16:

- [x] Name chosen and availability-verified (crates.io, npm, GitHub, domains)
- [x] Architecture and crate choices decided (below)
- [x] This file
- [x] Git repo initialised, `main`
- [x] Git identity configured globally as `darthrax`
- [x] Rust toolchain installed and smoke-tested
- [x] `gh` installed and **authenticated** as `darthrax` (SSH), org `theitlab-dev` visible
- [x] GitHub repo created and pushed — public, topics set
- [x] Crate scaffolded: `Cargo.toml`, `src/main.rs`, `README.md`, `LICENSE`; `fmt`, `clippy`, build all clean
- [x] **`gyroid` claimed on crates.io** — `v0.0.1` published 2026-11-16 after the
      maintainer verified the account email. Tagged `v0.0.1`, GitHub release created.
      Verified end to end: `cargo install gyroid` from the registry runs.
- [ ] Real slicing: mesh load, layer plan, infill, G-code emit
- [ ] `serve` / `slice` CLI — everything else

### Confirmed environment (the dev box)

This repo lives **on the printer host** — a Raspberry Pi 4.

| | |
|---|---|
| Arch / OS | `aarch64` / Debian 12 (bookworm) |
| Hardware | 4 cores, 7.5 GB RAM, 46 GB free (SD card) |
| Moonraker | **live** at `http://localhost:7125` — `GET /server/info` returns 200 |
| Klipper / Mainsail | installed (`~/klipper`, `~/mainsail`, `~/printer_data`) |
| rustc / cargo | **1.100.0 stable** (2026-11-09 / 2026-11-05), via rustup |
| `gh` | 2.23.0 (Debian package) |
| cc / pkg-config | gcc 12.2.0, pkg-config present; `libssl-dev` installed |
| Node / npm | v22.23.4 / 10.9.9 (only needed if the UI gets a build step) |
| Smoke test | `cargo build` of a hello-world: **~4 s** debug |

**Toolchain location:** `~/.cargo/bin`. rustup was installed with `--no-modify-path`, so
PATH is set up manually in three places: a source block in `~/.bashrc` (interactive) and
`~/.profile` (login), plus symlinks in `~/.local/bin`. Verified working in both
interactive and non-interactive login shells. In a bare non-login shell, either
`export PATH="$HOME/.cargo/bin:$PATH"` first or call `~/.cargo/bin/cargo` explicitly.

**Performance reality:** 4 cores + SD card. `cargo check` and debug builds are fine.
Full `--release` builds with heavy dependencies will be slow and disk-heavy; prefer CI
for distribution binaries. Use `lld` if linking becomes a bottleneck; do not add `mold`
on this box without asking.

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

Verified present and maintained on crates.io 2026-11-16 (downloads / last update).

| Concern | Choice | Notes |
|---|---|---|
| HTTP server | `axum` + `tokio` | 595 M dl — serve UI + API on one port |
| Static UI | `rust-embed` | **must** embed; no external `dist/` at runtime (68 M dl) |
| Moonraker WS | `tokio-tungstenite` | `/websocket` for live status (363 M dl) |
| Moonraker REST | `reqwest` | upload, start, query; **use `rustls`, not OpenSSL** (see below) |
| 2D booleans / offsetting | **`clipper2`** | 455 k dl, updated 2026-11-12 — safe Rust API over Clipper2, the *same* clipper family real slicers use. **Primary geometry workhorse.** |
| 2D alternative | `i_overlay` | 12 M dl, pure Rust, actively updated — fallback/benchmark against clipper2 |
| Math | `nalgebra` | 102 M dl — transforms, vectors |
| 3D collision | `parry3d` | 3.3 M dl — support checks, later |
| Mesh IO | `stl_io` (5.5 M dl), `obj-rs`, 3MF via `quick-xml` + `zip` | 3MF is a zip of XML |
| G-code validation | `gcode` | 30 k dl — parse/no-std, useful for testing our own output |
| Persistence | `rusqlite` (`bundled`) | 156 M dl — jobs, history, settings |
| Errors | `thiserror` (lib), `anyhow` (bin) | no `unwrap()` in library code |
| Logging | `tracing` + `tracing-subscriber` | structured, stderr |
| CLI | `clap` | `gyroid serve`, `gyroid slice`, etc. |

Use `rustls` rather than native OpenSSL in `reqwest` — native TLS is the single most
common way a "single binary" turns into a binary plus system packages.

**On the geometry risk** (revised 2026-11-16): there is still no mature Rust *slicer* to
build on — `cleave` ("from-scratch geometry kernel for CAD") is abandoned at 28
downloads, `imlet` (implicit modelling) last moved 2025-07, and
`cortenforge-mesh-lattice` has 53 downloads. **But** `clipper2` gives us the hardest
primitive — robust polygon offsetting and boolean ops — as a maintained, idiomatic Rust
crate. The real work left is layer planning, infill generation, and the G-code emitter,
not polygon algebra. Scope the first milestone to "slice a simple STL with 2–3 infill
patterns and preview it", not "OrcaSlicer parity".

---

## 5. Commands

PATH caveat in §3. Once scaffolding lands:

```bash
cargo run -- serve --moonraker http://localhost:7125   # dev server, open :8080
cargo run -- slice model.stl -o model.gcode            # headless slice
cargo check                                            # fast feedback
cargo clippy --all-targets -- -D warnings              # must be clean
cargo fmt --all                                        # always before commit
cargo test                                             # unit tests
cargo build --release                                  # slow on this box
```

UI (only if a JS build step is introduced — currently undecided, §9):

```bash
npm --prefix ui install
npm --prefix ui run build    # output must be embedded into the binary
```

Git / release:

```bash
gh auth status                                  # done: logged in as darthrax (SSH)
git push                                        # remote: git@github.com:theitlab-dev/gyroid.git
cargo login                                     # done
cargo publish                                   # done: v0.0.1 published 2026-11-16
git tag -a vX.Y.Z -m "..." && git push origin vX.Y.Z
gh release create vX.Y.Z --generate-notes
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
3. Never write to `printer_data/config/` — that is Moonraker's / Mainsail's territory.
4. Any G-code we emit must be reviewable in the UI before upload.
5. Destructive actions (restart, emergency stop) require a distinct, non-default control.
6. Fail loudly and read-only when Moonraker is unreachable — never silently retry a print.

---

## 7. Conventions

- `rustfmt` defaults, `clippy -D warnings` in CI.
- Library modules return typed errors (`thiserror`); `anyhow` only at the binary edge.
- No `unwrap()`/`expect()` in library code. In tests it is fine.
- Public items get doc comments. Modules get a one-line `//!` header.
- Commit as **conventional commits** (`feat:`, `fix:`, `docs:`, `refactor:`, `chore:`),
  authored as `darthrax <1251533+darthrax@users.noreply.github.com>`.
- Any new dependency needs a one-line justification in the commit body — especially
  anything that pulls in C, or anything that breaks the single-binary goal.
- Do not commit secrets. crates.io tokens live in `~/.cargo/credentials.toml` and GitHub
  auth in `gh`'s keyring — **never** in this repo, and never paste a token into a chat
  or a commit. Printer credentials stay in local config, not source.

---

## 8. Maintaining this file

This file is **not** write-once. It is the project's memory, and keeping it true is part
of every task — treat staleness here as a bug.

### Update it when any of these happen

| Trigger | Which section |
|---|---|
| A crate is added, removed or replaced | §4 table |
| A build/run/test command changes | §5 |
| Toolchain / OS / hardware changes on the dev box | §3 |
| Moonraker endpoint or behaviour is discovered to differ | §6 |
| A design decision is made or reversed | §2, §9, and the log |
| The project moves from pre-scaffold to working | §1 status, §3 checklist |
| License, maintainer or author identity is decided | §1, §2, §9 |
| A convention is added or dropped | §7 |
| Repo layout changes materially | §4 |
| A milestone is completed | §3 checklist, §10 |

### Rules for editing this file

1. **Edit in place, keep it short.** This is an overview, not documentation. If a topic
   outgrows its section, move it to `docs/` and leave a one-line pointer here.
2. **Never delete the rationale.** If a decision is reversed, keep the old reasoning in
   the entry and say why it changed. Future agents repeat mistakes without it.
3. **Mark uncertainty explicitly** — "TBD", "unverified", "as of <date>". Never state a
   guess as fact.
4. **Verify, don't assume.** Commands, endpoints, crate versions, and availability claims
   should be checked against reality before being recorded.
5. **Date every factual claim that can rot** (tool versions, availability checks,
   observed API behaviour).
6. **Append to the update log** below with a one-line summary. Do not rewrite history.
7. If you are unsure whether a change belongs here, add it to
   [Open questions](#9-open-questions) instead of guessing.

### Update log

| Date | Change |
|---|---|
| 2026-11-16 | **`gyroid` claimed on crates.io.** `v0.0.1` published after the maintainer verified the account email; tagged and released. `cargo install gyroid` verified from the registry. Publish blocker from the previous entry is resolved. |
| 2026-11-16 | Crate scaffolded (`Cargo.toml`, `src/main.rs`, `README.md`, `LICENSE`), GitHub repo created and pushed to `theitlab-dev/gyroid`. `cargo publish 0.0.1` **rejected** — crates.io requires a verified email. Name still free. License set to MIT as a default, pending maintainer confirmation. |
| 2026-11-16 | File created. Name locked to `gyroid` after availability sweep (crates.io, npm, GitHub, domains). Architecture and crate choices recorded. Dev box confirmed as Klipper/RPi host with no Rust toolchain. |
| 2026-11-16 | Identity locked (`darthrax`, `theitlab-dev`). rustup/rustc 1.100.0, `gh` 2.23.0, and build deps installed; smoke-tested. Git initialised with one commit. Crates verified on crates.io; `clipper2` promoted to primary geometry choice, replacing the earlier "no Rust geometry exists" pessimism. |

---

## 9. Open questions

Answer these before or during scaffolding, and record the answers above.

1. **Which "very old software" is this a modern remake of?** The design stance in §1 says
   from-scratch remake, but the ancestor(s) are still unnamed. This drives feature scope
   more than anything else — settle it first.
2. **Confirm or override the licence.** MIT was chosen unilaterally as the low-risk
   default so scaffolding could proceed. Raise it now if you wanted Apache-2.0 (patent
   grant), dual MIT/Apache-2.0 (Rust convention), or AGPL. Changing it before the first
   real release is cheap; after `0.1.0` it is not.
3. **UI approach:** hand-written HTML/JS embedded directly, or a JS build step
   (Vite/Svelte/React) whose output is `rust-embed`ed? Affects §5 and the
   single-binary rule.
4. **Target platforms for v0.1** — Linux only, or Linux + macOS + Windows from the
   start? Affects CI and how much platform abstraction we build now.

---

## 10. Ready to develop

Environment and scaffold are in place; the repo is pushed. To start a development
session:

1. Open a **new login shell** (so `~/.bashrc` puts `~/.cargo/bin` on PATH), then
   `cd ~/gyroid`.
2. Answer **§9.1** (which old software this remakes) and **§9.2** (licence) — they shape
   the first milestone.
3. Build the `serve` / `slice` CLI skeleton, then the slicer core per §4.

Auth is already configured (`gh` over SSH, `cargo` logged in). Credentials live in
`~/.cargo/credentials.toml` and `gh`'s config — never in this repo, never in a chat.
