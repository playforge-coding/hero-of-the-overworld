//! Entry point. macroquad's `main` macro sets up the window and GL context
//! (native and web) and drives the async game loop in [`hero_of_the_overworld::run`].

use hero_of_the_overworld::{run, window_conf};

#[macroquad::main(window_conf)]
async fn main() {
    run().await;
}
