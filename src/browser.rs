use anyhow::{anyhow, Result};
use web_sys::{Window, Document, HtmlCanvasElement, CanvasRenderingContext2d};

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
