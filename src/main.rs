//! Native binary entry point. The web build uses the `wasm_start` export in
//! `lib.rs` instead (see `index.html` / trunk).

fn main() {
    hero_of_the_overworld::start();
}
