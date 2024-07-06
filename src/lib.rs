use std::rc::Rc;
use std::sync::Mutex;

use rand::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use serde::Deserialize;
use std::collections::HashMap;
use gloo_utils::format::JsValueSerdeExt;

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


fn draw_triangle(
    context: &web_sys::CanvasRenderingContext2d,
    points: [(f64, f64); 3],
    color: (u8, u8, u8),
) {
    let [top, left, right] = points;

    let color_str = format!("rgb({},{},{})", color.0, color.1, color.2);
    context.set_fill_style(&wasm_bindgen::JsValue::from_str(&color_str));

    context.move_to(top.0, top.1);
    context.begin_path();
    context.line_to(left.0, left.1);
    context.line_to(right.0, right.1);
    context.line_to(top.0, top.1);
    context.close_path();
    context.stroke();
    context.fill();
}

fn sierpinsk(
    context: &web_sys::CanvasRenderingContext2d,
    points: [(f64, f64); 3],
    color: (u8, u8, u8),
    depth: u8,
) {
    if depth == 0 {
        draw_triangle(context, points, color);
    } else {
        let mut rng = rand::thread_rng();
        let next_color = (
            rng.gen_range(0..255),
            rng.gen_range(0..255),
            rng.gen_range(0..255),
        );

        let [top, left, right] = points;
        let left_mid = ((top.0 + left.0) / 2.0, (top.1 + left.1) / 2.0);
        let right_mid = ((top.0 + right.0) / 2.0, (top.1 + right.1) / 2.0);
        let bottom_mid = ((left.0 + right.0) / 2.0, (left.1 + right.1) / 2.0);

        sierpinsk(context, [top, left_mid, right_mid], next_color, depth - 1);
        sierpinsk(context, [left_mid, left, bottom_mid], next_color, depth - 1);
        sierpinsk(
            context,
            [right_mid, bottom_mid, right],
            next_color,
            depth - 1,
        );
    }
}
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();


    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document
        .get_element_by_id("canvas")
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap();
    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();
    wasm_bindgen_futures::spawn_local(async move {
        let (success_tx, success_rx) = futures::channel::oneshot::channel::<Result<(), JsValue>>();
        let success_tx = Rc::new(Mutex::new(Some(success_tx)));
        let error_tx = Rc::clone(&success_tx);

        let image = web_sys::HtmlImageElement::new().unwrap();
        let callback = Closure::once(move||{
            if let Some(success_tx) = success_tx.lock().ok().and_then(|mut opt| opt.take()) {
                success_tx.send(Ok(()));
            }
        });
        let error_callback = Closure::once(move|err|{
            if let Some(error_tx) = error_tx.lock().ok().and_then(|mut opt| opt.take()) {
                error_tx.send(Err(err));
            }
        });
        image.set_onload(Some(callback.as_ref().unchecked_ref()));
        image.set_onerror(Some(error_callback.as_ref().unchecked_ref()));
        image.set_src("Idle (1).png");
        success_rx.await;
        context.draw_image_with_html_image_element(&image, 0.0, 0.0);
        sierpinsk(
            &context,
            [(300.0, 0.0), (0.0, 600.0), (600.0, 600.0)],
            (0, 255, 0),
            5,
        );
        // JSONファイルをロードする
        let json = fetch_json("rhb.json").await.expect("Failed to fetch JSON");
        // JSONファイルをパースしてRustの構造体にする
        let sheet: Sheet = json.into_serde().expect("Failed to parse JSON");
        // 画像をHtmlImageElementに読み込む
        let (success_tx, success_rx) = futures::channel::oneshot::channel::<Result<(), JsValue>>();
        let success_tx = Rc::new(Mutex::new(Some(success_tx)));
        let error_tx = Rc::clone(&success_tx);

        let image = web_sys::HtmlImageElement::new().unwrap();
        let callback = Closure::once(move||{
            if let Some(success_tx) = success_tx.lock().ok().and_then(|mut opt| opt.take()) {
                success_tx.send(Ok(()));
            }
        });
        let error_callback = Closure::once(move|err|{
            if let Some(error_tx) = error_tx.lock().ok().and_then(|mut opt| opt.take()) {
                error_tx.send(Err(err));
            }
        });
        image.set_onload(Some(callback.as_ref().unchecked_ref()));
        image.set_onerror(Some(error_callback.as_ref().unchecked_ref()));
        image.set_src("rhb.png");
        success_rx.await;
        // 画像エレメントの一部だけを表示するようにしたバージョンのdrawImageを用いる
    });


    Ok(())
}

async fn fetch_json(json_path: &str) -> Result<JsValue, JsValue> {
    let window = web_sys::window().unwrap();
    let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_str(json_path)).await?;
    let resp: web_sys::Response = resp_value.dyn_into().unwrap();

    wasm_bindgen_futures::JsFuture::from(resp.json()?).await
}