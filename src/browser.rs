use anyhow::{anyhow, Result};
use web_sys::{Window, Document, HtmlCanvasElement, CanvasRenderingContext2d};
use wasm_bindgen::JsCast;

macro_rules! log {
    ($($t:tt)*) => {
        web_sys::console::log_1(&format!($($t)*).into());
    };
}

pub fn window() -> Result<Window>{
    web_sys::window().ok_or_else(|| anyhow!("No Window Founc"))
}

pub fn document() -> Result<Document>{
    window()?.document().ok_or_else(|| anyhow!("No Document Founc"))
}

pub fn canvas() -> Result<HtmlCanvasElement>{
    document()?
        .get_element_by_id("canvas") // "canvas"がハードコーディングされているが必要になるまで再設定可能にするのはやめる
        .ok_or_else(|| anyhow!("No Canvas Element Found with ID 'canvas'"))?
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|element| anyhow!("Error convrting {:#?} to HtmlCanvasElement", element))
}
