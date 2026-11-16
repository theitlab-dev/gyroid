//! Gyroid — a modern web-based slicer with direct Moonraker integration.
//!
//! This is a placeholder entry point published to reserve the `gyroid` crate
//! name. The real CLI (`serve`, `slice`) lands in the next milestone.
//!
//! See `AGENTS.md` for architecture, conventions and current status.

/// Project homepage.
const HOMEPAGE: &str = "https://gyroid.theitlab.dev";

/// Repository.
const REPOSITORY: &str = "https://github.com/theitlab-dev/gyroid";

fn main() {
    println!("gyroid {} — not yet implemented", env!("CARGO_PKG_VERSION"));
    println!("  homepage:   {HOMEPAGE}");
    println!("  repository: {REPOSITORY}");
}
