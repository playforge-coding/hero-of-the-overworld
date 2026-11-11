//! Rebuild whenever the embedded content tree changes.
//!
//! `rust-embed` bakes `assets/data/` (the split content database: one RON file
//! per entity plus the CSV tilemaps) and the house tileset into the binary at
//! compile time. The derive only creates rebuild dependencies on the files that
//! existed when it last expanded, so **adding or removing** a file wouldn't
//! otherwise trigger a rebuild. Watching the directories closes that gap, so
//! `cargo build` always embeds the current content.
fn main() {
    println!("cargo::rerun-if-changed=assets/data");
    println!("cargo::rerun-if-changed=assets/textures/tiles/house");
}
