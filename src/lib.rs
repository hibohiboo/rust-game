#[macro_use]
mod browser;
mod engine;
use std::rc::Rc;
use std::sync::Mutex;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use gloo_utils::format::JsValueSerdeExt;
use serde::Deserialize;
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

    let context = browser::context().expect("Failed to get canvas context");

    browser::spawn_local(async move {
        let sheet: Sheet = browser::fetch_json("rhb.json").await.expect("Count not fetch rhb.json").into_serde().expect("Could not parse rhb.json");
        // 画像をHtmlImageElementに読み込む
        let (success_tx, success_rx) = futures::channel::oneshot::channel::<Result<(), JsValue>>();
        let success_tx = Rc::new(Mutex::new(Some(success_tx)));
        let error_tx = Rc::clone(&success_tx);

        let image = web_sys::HtmlImageElement::new().unwrap();
        let callback = Closure::once(move || {
            if let Some(success_tx) = success_tx.lock().ok().and_then(|mut opt| opt.take()) {
                success_tx.send(Ok(()));
            }
        });
        let error_callback = Closure::once(move |err| {
            if let Some(error_tx) = error_tx.lock().ok().and_then(|mut opt| opt.take()) {
                error_tx.send(Err(err));
            }
        });
        image.set_onload(Some(callback.as_ref().unchecked_ref()));
        image.set_onerror(Some(error_callback.as_ref().unchecked_ref()));
        image.set_src("rhb.png");
        success_rx.await;

        let mut frame = -1;
        let interval_callback = Closure::wrap(Box::new(move || {
            frame = (frame + 1) % 8;
            let frame_name = format!("Run ({}).png", frame + 1);
            context.clear_rect(0.0, 0.0, 600.0, 600.0);
            // 画像エレメントの一部だけを表示するようにしたバージョンのdrawImageを用いる
            let sprite = sheet
                .frames
                .get(&frame_name)
                .expect("Cell not found in sheet");
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
        }) as Box<dyn FnMut()>);
        browser::window().unwrap().set_interval_with_callback_and_timeout_and_arguments_0(
            interval_callback.as_ref().unchecked_ref(),
            50,
        );
        interval_callback.forget();
    });

    Ok(())
}

async fn fetch_json(json_path: &str) -> Result<JsValue, JsValue> {
    let window = web_sys::window().unwrap();
    let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_str(json_path)).await?;
    let resp: web_sys::Response = resp_value.dyn_into().unwrap();

    wasm_bindgen_futures::JsFuture::from(resp.json()?).await
}
