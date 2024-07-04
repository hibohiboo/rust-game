use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

fn draw_triangle(context: &web_sys::CanvasRenderingContext2d, points: [(f64,f64);3]) {
  let [top, left, right] = points;

    context.move_to(top.0, top.1);
    context.begin_path();
    context.line_to(left.0, left.1);
    context.line_to(right.0, right.1);
    context.line_to(top.0, top.1);
    context.close_path();
    context.stroke();
}

fn sierpinsk(context: &web_sys::CanvasRenderingContext2d, points: [(f64,f64);3], depth: u8) {
  if depth == 0 {
    draw_triangle(context, points);
  } else {
    let [top, left, right] = points;
    let left_mid = ((top.0 + left.0) / 2.0, (top.1 + left.1) / 2.0);
    let right_mid = ((top.0 + right.0) / 2.0, (top.1 + right.1) / 2.0);
    let bottom_mid = ((left.0 + right.0) / 2.0, (left.1 + right.1) / 2.0);

    sierpinsk(context, [top, left_mid, right_mid], depth - 1);
    sierpinsk(context, [left_mid, left, bottom_mid], depth - 1);
    sierpinsk(context, [right_mid, bottom_mid, right], depth - 1);
  }
}
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap().dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
    let context = canvas.get_context("2d").unwrap().unwrap().dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap();
    
    sierpinsk(&context, [(300.0, 0.0), (0.0, 600.0), (600.0, 600.0)],5);


    
    Ok(())
}