use anyhow::{anyhow, Result};

use js_sys::ArrayBuffer;
use wasm_bindgen::{closure::WasmClosure, closure::WasmClosureFnOnce, prelude::Closure, JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{CanvasRenderingContext2d, Document, HtmlCanvasElement, HtmlElement, Response, Window};

macro_rules! log {
    ($($t:tt)*) => {
        web_sys::console::log_1(&format!($($t)*).into());
    };
}
macro_rules! error {
    ( $( $t:tt )* ) => {
        web_sys::console::error_1(&format!( $( $t )* ).into());
    }
}

pub fn window() -> Result<Window> {
  web_sys::window().ok_or_else(|| anyhow!("No Window Founc"))
}

pub fn document() -> Result<Document> {
  window()?
    .document()
    .ok_or_else(|| anyhow!("No Document Founc"))
}

pub fn canvas() -> Result<HtmlCanvasElement> {
  document()?
    .get_element_by_id("canvas") // "canvas"がハードコーディングされているが必要になるまで再設定可能にするのはやめる
    .ok_or_else(|| anyhow!("No Canvas Element Found with ID 'canvas'"))?
    .dyn_into::<web_sys::HtmlCanvasElement>()
    .map_err(|element| anyhow!("Error convrting {:#?} to HtmlCanvasElement", element))
}
pub fn context() -> Result<CanvasRenderingContext2d> {
  canvas()?
    .get_context("2d")
    .map_err(|js_value| anyhow!("Error getting 2d context {:#?}", js_value))?
    .ok_or_else(|| anyhow!("Failed to get 2d context"))?
    .dyn_into::<web_sys::CanvasRenderingContext2d>()
    .map_err(|context| {
      anyhow!(
        "Error converting {:#?} to CanvasRenderingContext2d",
        context
      )
    })
}

pub fn spawn_local<F>(future: F)
where
  F: std::future::Future<Output = ()> + 'static,
{
  wasm_bindgen_futures::spawn_local(future);
}

pub async fn fetch_with_str(resource: &str) -> Result<JsValue> {
  JsFuture::from(window()?.fetch_with_str(resource))
    .await
    .map_err(|err| anyhow!("Failed to fetch resource: {:#?}", err))
}

pub async fn fetch_response(resource: &str) -> Result<Response> {
  let response = fetch_with_str(resource).await?;
  response
    .dyn_into()
    .map_err(|element| anyhow!("Error converting {:#?} to Response", element))
}
pub async fn fetch_array_buffer(resource: &str) -> Result<ArrayBuffer> {
  let response = fetch_response(resource).await?;
  let array_buffer = response
    .array_buffer()
    .map_err(|err| anyhow!("Failed to get array buffer: {:#?}", err))?;
  JsFuture::from(array_buffer)
    .await
    .map_err(|err| anyhow!("Failed to parse array buffer: {:#?}", err))?
    .dyn_into()
    .map_err(|element| anyhow!("Error converting {:#?} to ArrayBuffer", element))
}
pub async fn fetch_json(json_path: &str) -> Result<JsValue> {
  let response = fetch_response(json_path).await?;
  let res: Response = response
    .dyn_into()
    .map_err(|element| anyhow!("Error converting {:#?} to Response", element))?;
  JsFuture::from(
    res
      .json()
      .map_err(|err| anyhow!("Failed to get JSON: {:#?}", err))?,
  )
  .await
  .map_err(|err| anyhow!("Failed to parse JSON: {:#?}", err))
}

pub fn new_image() -> Result<web_sys::HtmlImageElement> {
  web_sys::HtmlImageElement::new().map_err(|err| anyhow!("Failed to create new image: {:#?}", err))
}

pub fn closure_once<F, A, R>(fn_once: F) -> Closure<F::FnMut>
where
  F: 'static + WasmClosureFnOnce<A, R>,
{
  Closure::once(fn_once)
}

pub fn closure_wrap<T: WasmClosure + ?Sized>(data: Box<T>) -> Closure<T> {
  Closure::wrap(data)
}
pub type LoopClosure = Closure<dyn FnMut(f64)>;
pub fn create_raf_closure(f: impl FnMut(f64) + 'static) -> LoopClosure {
  closure_wrap(Box::new(f))
}
pub fn request_animation_frame(callback: &LoopClosure) -> Result<i32> {
  window()?
    .request_animation_frame(callback.as_ref().unchecked_ref())
    .map_err(|err| anyhow!("Failed to request animation frame: {:#?}", err))
}

pub fn now() -> Result<f64> {
  Ok(
    window()?
      .performance()
      .ok_or_else(|| anyhow!("Performance object not found"))?
      .now(),
  )
}

pub fn draw_ui(html: &str) -> Result<()> {
  find_ui().and_then(|ui| {
    ui.insert_adjacent_html("afterbegin", html)
      .map_err(|err| anyhow!("Failed to insert HTML: {:#?}", err))
  })
}

pub fn hide_ui() -> Result<()> {
  let ui = find_ui()?;
  if let Some(child) = ui.first_child() {
    ui.remove_child(&child)
      .map(|_removed_child| ())
      .map_err(|err| anyhow!("Failed to remove child: {:#?}", err))
      .and_then(|_unit| {
        canvas()?
          .focus()
          .map_err(|err| anyhow!("Failed to focus canvas: {:#?}", err))
      })
  } else {
    Ok(())
  }
}
fn find_ui() -> Result<web_sys::Element> {
  document()?
    .get_element_by_id("ui")
    .ok_or_else(|| anyhow!("No UI Element Found with ID 'ui'"))
}

pub fn find_element_by_id(id: &str) -> Result<HtmlElement> {
  document().and_then(|doc| {
    doc
      .get_element_by_id(id)
      .ok_or_else(|| anyhow!("No Element Found with ID '{}'", id))
      .and_then(|element| {
        element
          .dyn_into::<web_sys::HtmlElement>()
          .map_err(|element| anyhow!("Error converting {:#?} to HtmlElement", element))
      })
  })
}
