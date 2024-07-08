#[macro_use]
mod browser;
mod engine;
mod game;

use engine::GameLoop;
use serde::Deserialize;
use wasm_bindgen::prelude::*;
use std::collections::HashMap;

#[derive(Deserialize)]
struct Rect {
    x: u64,
    y: u64,
    w: u64,
    h: u64,
}

#[derive(Deserialize)]
struct Cell {
    frame: Rect,
}

#[derive(Deserialize)]
struct Sheet {
    frames: HashMap<String, Cell>,
}

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    browser::spawn_local(async move {
        let game = game::WalkTheDog::new();
        GameLoop::start(game).await.expect("Could not start game loop");
    });
    Ok(())
}
