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
    let interval_callback = Closure::wrap(Box::new(move || { }) as Box<dyn FnMut()>);
    window.set_interval_with_callback_and_timeout_and_arguments_0(
            interval_callback.as_ref().unchecked_ref(),
            50,
        );
    interval_callback.forget();
    wasm_bindgen_futures::spawn_local(async move {

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
        let sprite = sheet.frames.get("Run (1).png").expect("Cell not found in sheet");
        context.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
            &image,
            sprite.frame.x as f64,
            sprite.frame.y as f64,
            sprite.frame.w as f64,
            sprite.frame.h as f64,
            300.0,
            300.0,
            sprite.frame.w as f64,
            sprite.frame.h as f64,
        );
    });


    Ok(())
}

async fn fetch_json(json_path: &str) -> Result<JsValue, JsValue> {
    let window = web_sys::window().unwrap();
    let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_str(json_path)).await?;
    let resp: web_sys::Response = resp_value.dyn_into().unwrap();

    wasm_bindgen_futures::JsFuture::from(resp.json()?).await
}