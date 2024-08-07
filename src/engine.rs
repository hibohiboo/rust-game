use crate::browser::LoopClosure;
use crate::browser::{self};
use crate::sound;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::channel::mpsc::{unbounded, UnboundedReceiver};
use futures::channel::oneshot::channel;
use serde::Deserialize;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Mutex;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlImageElement;
use web_sys::{CanvasRenderingContext2d, HtmlElement};
// use wasm_bindgen_test::__rt::browser;

#[derive(Clone, Copy, Default)]
pub struct Point {
  pub x: i16,
  pub y: i16,
}

#[derive(Deserialize, Clone)]
pub struct SheetRect {
  pub x: i16,
  pub y: i16,
  pub w: i16,
  pub h: i16,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Cell {
  pub frame: SheetRect,
  pub sprite_source_size: SheetRect,
}

#[derive(Deserialize, Clone)]
pub struct Sheet {
  pub frames: HashMap<String, Cell>,
}

#[derive(Default)]
pub struct Rect {
  pub position: Point,
  pub width: i16,
  pub height: i16,
}

impl Rect {
  pub const fn new(position: Point, width: i16, height: i16) -> Self {
    Rect {
      position,
      width,
      height,
    }
  }

  pub const fn new_from_x_y(x: i16, y: i16, width: i16, height: i16) -> Self {
    Rect::new(Point { x, y }, width, height)
  }

  pub fn intersects(&self, rect: &Rect) -> bool {
    self.x() < rect.right() && self.right() > rect.x() && self.y() < rect.bottom() && self.bottom() > rect.y()
  }

  pub fn right(&self) -> i16 {
    self.x() + self.width
  }

  pub fn bottom(&self) -> i16 {
    self.y() + self.height
  }

  pub fn set_x(&mut self, x: i16) {
    self.position.x = x
  }

  pub fn x(&self) -> i16 {
    self.position.x
  }

  pub fn y(&self) -> i16 {
    self.position.y
  }
}

#[cfg(test)] // Only compile the following module when running tests
mod tests {
  // 他のコードから隔離するため mod キーワードでモジュール化する
  use super::*;

  #[test]
  fn two_rects_that_intersect_on_the_left() {
    let rect1 = Rect {
      position: Point { x: 10, y: 10 },
      height: 100,
      width: 100,
    };

    let rect2 = Rect {
      position: Point { x: 0, y: 10 },
      height: 100,
      width: 100,
    };

    assert_eq!(rect2.intersects(&rect1), true);
  }

  // 四角形が上から重なっている場合
  #[test]
  fn two_rects_that_intersect_on_the_top() {
    let rect1 = Rect {
      position: Point { x: 10, y: 10 },
      height: 100,
      width: 100,
    };

    let rect2 = Rect {
      position: Point { x: 10, y: 0 },
      height: 100,
      width: 100,
    };

    assert_eq!(rect2.intersects(&rect1), true);
  }
  // 四角形が右から重なっている場合
  #[test]
  fn two_rects_that_intersect_on_the_right() {
    let rect1 = Rect {
      position: Point { x: 10, y: 10 },
      height: 100,
      width: 100,
    };

    let rect2 = Rect {
      position: Point { x: 10, y: 10 },
      height: 100,
      width: 100,
    };

    assert_eq!(rect2.intersects(&rect1), true);
  }
  // 四角形が重なっていない場合
  #[test]
  fn two_rects_that_do_not_intersect() {
    let rect1 = Rect {
      position: Point { x: 10, y: 10 },
      height: 100,
      width: 100,
    };

    let rect2 = Rect {
      position: Point { x: 200, y: 200 },
      height: 100,
      width: 100,
    };

    assert_eq!(rect2.intersects(&rect1), false);
  }
}

pub async fn load_image(source: &str) -> Result<HtmlImageElement> {
  let image = browser::new_image()?;

  let (complete_tx, complete_rx) = channel::<Result<()>>();
  let success_tx = Rc::new(Mutex::new(Some(complete_tx)));
  let error_tx = Rc::clone(&success_tx);
  let success_callback = browser::closure_once(move || {
    if let Some(success_tx) = success_tx.lock().ok().and_then(|mut opt| opt.take()) {
      if let Err(err) = success_tx.send(Ok(())) {
        error!("Could not send successful image loaded message! {:#?}", err);
      }
    }
  });

  let error_callback: Closure<dyn FnMut(JsValue)> = browser::closure_once(move |err| {
    if let Some(error_tx) = error_tx.lock().ok().and_then(|mut opt| opt.take()) {
      if let Err(err) = error_tx.send(Err(anyhow!("Error Loading Image: {:#?}", err))) {
        error!("Could not send error message on loading image! {:#?}", err);
      }
    }
  });

  image.set_onload(Some(success_callback.as_ref().unchecked_ref()));
  image.set_onerror(Some(error_callback.as_ref().unchecked_ref()));
  image.set_src(source);

  complete_rx.await??;

  Ok(image)
}

#[async_trait(?Send)]
pub trait Game {
  async fn initialize(&self) -> Result<Box<dyn Game>>;
  fn update(&mut self, keystate: &KeyState);
  fn draw(&self, renderer: &Renderer);
}

const FRAME_SIZE: f32 = 1.0 / 60.0 * 1000.0;
pub struct GameLoop {
  last_frame: f64,
  accumulated_delta: f32,
}
type SharedLoopClosure = Rc<RefCell<Option<LoopClosure>>>;

impl GameLoop {
  pub async fn start(game: impl Game + 'static) -> Result<()> {
    let mut keyevent_receiver = prepare_input()?;
    let mut game = game.initialize().await?;
    let mut game_loop = GameLoop {
      last_frame: browser::now()?,
      accumulated_delta: 0.0,
    };

    let renderer = Renderer {
      context: browser::context()?,
    };

    let f: SharedLoopClosure = Rc::new(RefCell::new(None));
    let g = f.clone();

    let mut keystate = KeyState::new();

    *g.borrow_mut() = Some(browser::create_raf_closure(move |perf: f64| {
      process_input(&mut keystate, &mut keyevent_receiver);
      game_loop.accumulated_delta += (perf - game_loop.last_frame) as f32;
      // game.drawに時間がかかると、updateが呼ばれる回数が減るため、その分を補填。描画を犠牲にして内部処理は確実に行うようにする。(drawを行わないupdateを行う)
      while game_loop.accumulated_delta > FRAME_SIZE {
        game.update(&keystate);
        game_loop.accumulated_delta -= FRAME_SIZE;
      }
      game_loop.last_frame = perf;
      game.draw(&renderer);

      let _ = browser::request_animation_frame(f.borrow().as_ref().unwrap());
    }));
    let _ = browser::request_animation_frame(
      g.borrow()
        .as_ref()
        .ok_or_else(|| anyhow!("GameLoop: Loop is None"))?,
    );
    Ok(())
  }
}

pub struct Renderer {
  context: CanvasRenderingContext2d,
}

impl Renderer {
  pub fn clear(&self, rect: &Rect) {
    self.context.clear_rect(
      rect.position.x.into(),
      rect.position.y.into(),
      rect.width.into(),
      rect.height.into(),
    );
  }
  pub fn draw_image(&self, image: &HtmlImageElement, frame: &Rect, destination: &Rect) {
    self
      .context
      .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
        image,
        frame.position.x.into(),
        frame.position.y.into(),
        frame.width.into(),
        frame.height.into(),
        destination.position.x.into(),
        destination.position.y.into(),
        destination.width.into(),
        destination.height.into(),
      )
      .expect("Drawing is thrown exceptions! Unrecoverable error.");
  }
  pub fn draw_entire_image(&self, image: &HtmlImageElement, position: Point) {
    self
      .context
      .draw_image_with_html_image_element_and_dw_and_dh(
        image,
        position.x.into(),
        position.y.into(),
        image.width().into(),
        image.height().into(),
      )
      .expect("Drawing is thrown exceptions! Unrecoverable error.");
  }
}
enum KeyPress {
  KeyUp(web_sys::KeyboardEvent),
  KeyDown(web_sys::KeyboardEvent),
}

pub struct KeyState {
  pressed_keys: HashMap<String, web_sys::KeyboardEvent>,
}
impl KeyState {
  pub fn new() -> Self {
    KeyState {
      pressed_keys: HashMap::new(),
    }
  }
  pub fn is_pressed(&self, code: &str) -> bool {
    self.pressed_keys.contains_key(code)
  }
  pub fn set_pressed(&mut self, code: &str, event: web_sys::KeyboardEvent) {
    self.pressed_keys.insert(code.into(), event);
  }
  pub fn set_released(&mut self, code: &str) {
    self.pressed_keys.remove(code);
  }
}
fn process_input(state: &mut KeyState, keyevent_receiver: &mut UnboundedReceiver<KeyPress>) {
  loop {
    match keyevent_receiver.try_next() {
      Ok(None) => break,
      Err(_) => break,
      Ok(Some(evt)) => match evt {
        KeyPress::KeyUp(evt) => state.set_released(&evt.code()),
        KeyPress::KeyDown(evt) => state.set_pressed(&evt.code(), evt),
      },
    }
  }
}

/**
 * Prepare input events
 * ※canvas要素にはtabIndex属性がついておりキーボードイベントを取得できる前提とする。
 */
fn prepare_input() -> Result<UnboundedReceiver<KeyPress>> {
  let (keydown_sender, keyevent_receiver) = unbounded();
  let keydown_sender = Rc::new(RefCell::new(keydown_sender));
  let keyup_sender = Rc::clone(&keydown_sender);

  let onkeydown = browser::closure_wrap(Box::new(move |keycode: web_sys::KeyboardEvent| {
    log!("{}", &format!("Key Down: {}", keycode.key()));
    let _ = keydown_sender
      .borrow_mut()
      .start_send(KeyPress::KeyDown(keycode));
  }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);

  let onkeyup = browser::closure_wrap(Box::new(move |keycode: web_sys::KeyboardEvent| {
    let _ = keyup_sender
      .borrow_mut()
      .start_send(KeyPress::KeyUp(keycode));
  }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);

  browser::canvas()
    .unwrap()
    .set_onkeydown(Some(onkeydown.as_ref().unchecked_ref()));
  browser::canvas()
    .unwrap()
    .set_onkeyup(Some(onkeyup.as_ref().unchecked_ref()));
  onkeydown.forget();
  onkeyup.forget();
  Ok(keyevent_receiver)
}

pub struct Image {
  element: HtmlImageElement,
  bounding_box: Rect,
}

impl Image {
  pub fn new(element: HtmlImageElement, position: Point) -> Self {
    let bounding_box = Rect::new_from_x_y(
      position.x,
      position.y,
      element.width() as i16,
      element.height() as i16,
    );
    Self {
      element,
      bounding_box,
    }
  }
  pub fn draw(&self, renderer: &Renderer) {
    renderer.draw_entire_image(&self.element, self.bounding_box.position);
  }
  pub fn bounding_box(&self) -> &Rect {
    &self.bounding_box
  }
  pub fn move_horizontally(&mut self, distance: i16) {
    self.set_x(self.bounding_box.position.x + distance);
  }

  pub fn set_x(&mut self, x: i16) {
    self.bounding_box.set_x(x);
    self.bounding_box.position.x = x;
  }
  pub fn right(&self) -> i16 {
    (self.bounding_box.x() + self.bounding_box.width) as i16
  }
}

pub struct SpriteSheet {
  image: HtmlImageElement,
  sheet: Sheet,
}

impl SpriteSheet {
  pub async fn new(sheet: Sheet, image: HtmlImageElement) -> Self {
    SpriteSheet { image, sheet }
  }
  pub fn cell(&self, name: &str) -> Option<&Cell> {
    self.sheet.frames.get(name)
  }

  pub fn draw(&self, renderer: &Renderer, source: &Rect, destination: &Rect) {
    renderer.draw_image(&self.image, source, destination);
  }
}

#[derive(Clone)]
pub struct Audio {
  context: web_sys::AudioContext,
}
impl Audio {
  pub fn new() -> Result<Self> {
    let context = sound::create_audo_context()?;
    Ok(Audio { context })
  }
  pub async fn load_sound(&self, path: &str) -> Result<Sound> {
    let array_buffer = browser::fetch_array_buffer(path).await?;
    let buffer = sound::decode_audio_data(&self.context, &array_buffer).await?;
    Ok(Sound { buffer })
  }
  pub fn play_sound(&self, sound: &Sound) -> Result<()> {
    sound::play_sound(&self.context, &sound.buffer, sound::LOOPING::NO)
  }
  pub fn play_looping_sound(&self, sound: &Sound) -> Result<()> {
    sound::play_sound(&self.context, &sound.buffer, sound::LOOPING::YES)
  }
}
#[derive(Clone)]
pub struct Sound {
  pub buffer: web_sys::AudioBuffer,
}

pub fn add_click_handler(element: HtmlElement) -> UnboundedReceiver<()> {
  let (mut click_sender, click_receiver) = unbounded();
  let on_click = browser::closure_wrap(Box::new(move || {
    if let Err(err) = click_sender.start_send(()) {
      error!("Could not send click message {:#?}", err);
    }
  }) as Box<dyn FnMut()>);
  element.set_onclick(Some(on_click.as_ref().unchecked_ref()));
  on_click.forget();
  click_receiver
}
