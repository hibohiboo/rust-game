use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::console;

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap().dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
    let context = canvas.get_context("2d").unwrap().unwrap().dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap();

    context.move_to(300.0, 0.0); // 上
    context.begin_path();
    context.line_to(0.0, 600.0); // 左下
    context.line_to(600.0, 600.0); // 右下
    context.line_to(300.0, 0.0); // 上に戻る
    context.close_path();
    context.stroke();
    context.fill();
    
    Ok(())
}
