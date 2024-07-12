#[macro_use]
mod browser;
mod engine;
mod game;
mod segments;

use engine::GameLoop;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
  console_error_panic_hook::set_once();
  browser::spawn_local(async move {
    let game = game::WalkTheDog::new();
    GameLoop::start(game)
      .await
      .expect("Could not start game loop");
  });
  Ok(())
}
