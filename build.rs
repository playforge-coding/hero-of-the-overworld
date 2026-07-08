//! Rebuild whenever the embedded content tree changes.
//!
//! [`include_dir!`] bakes `assets/data/` (the split content database — one RON
//! file per entity plus the CSV tilemaps) into the binary at compile time. The
//! macro only creates rebuild dependencies on the files that existed when it last
//! expanded, so **adding or removing** a data file wouldn't otherwise trigger a
//! rebuild. Watching the directory closes that gap, so `cargo build` always
//! re-embeds the current content.
fn main() {
    println!("cargo:rerun-if-changed=assets/data");
}
