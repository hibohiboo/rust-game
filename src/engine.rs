use crate::browser::LoopClosure;
use crate::browser::{self};
use web_sys::CanvasRenderingContext2d;
use web_sys::HtmlImageElement;
use anyhow::{anyhow, Result};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Mutex;
use futures::channel::oneshot::channel;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
// use wasm_bindgen_test::__rt::browser;
use async_trait::async_trait;

#[derive(Clone, Copy, Default)]
pub struct Point {
    pub x: i16,
    pub y: i16,
}

pub struct Rect {
    pub x:f32,
    pub y:f32,
    pub width:f32,
    pub height:f32,
}
// #[derive(Default)]
// pub struct Rect {
//     pub position: Point,
//     pub width: i16,
//     pub height: i16,
// }

// impl Rect {
//     pub const fn new(position: Point, width: i16, height: i16) -> Self {
//         Rect {
//             position,
//             width,
//             height,
//         }
//     }

//     pub const fn new_from_x_y(x: i16, y: i16, width: i16, height: i16) -> Self {
//         Rect::new(Point { x, y }, width, height)
//     }

//     pub fn intersects(&self, rect: &Rect) -> bool {
//         self.x < rect.right()
//             && self.right() > rect.x
//             && self.y() < rect.bottom()
//             && self.bottom() > rect.y()
//     }

//     pub fn right(&self) -> i16 {
//         self.x + self.width
//     }

//     pub fn bottom(&self) -> i16 {
//         self.y() + self.height
//     }

//     pub fn set_x(&mut self, x: i16) {
//         self.position.x = x
//     }

//     pub fn x(&self) -> i16 {
//         self.position.x
//     }

//     pub fn y(&self) -> i16 {
//         self.position.y
//     }
// }

// #[cfg(test)] // Only compile the following module when running tests
// mod tests { // 他のコードから隔離するため mod キーワードでモジュール化する
//     use super::*;

//     #[test]
//     fn two_rects_that_intersect_on_the_left() {
//         let rect1 = Rect {
//             position: Point { x: 10, y: 10 },
//             height: 100,
//             width: 100,
//         };

//         let rect2 = Rect {
//             position: Point { x: 0, y: 10 },
//             height: 100,
//             width: 100,
//         };

//         assert_eq!(rect2.intersects(&rect1), true);
//     }

//     // 四角形が上から重なっている場合
//     #[test]
//     fn two_rects_that_intersect_on_the_top() {
//         let rect1 = Rect {
//             position: Point { x: 10, y: 10 },
//             height: 100,
//             width: 100,
//         };

//         let rect2 = Rect {
//             position: Point { x: 10, y: 0 },
//             height: 100,
//             width: 100,
//         };

//         assert_eq!(rect2.intersects(&rect1), true);
//     }
//     // 四角形が右から重なっている場合
//     #[test]
//     fn two_rects_that_intersect_on_the_right() {
//         let rect1 = Rect {
//             position: Point { x: 10, y: 10 },
//             height: 100,
//             width: 100,
//         };

//         let rect2 = Rect {
//             position: Point { x: 10, y: 10 },
//             height: 100,
//             width: 100,
//         };

//         assert_eq!(rect2.intersects(&rect1), true);
//     }
//     // 四角形が重なっていない場合
//     #[test]
//     fn two_rects_that_do_not_intersect() {
//         let rect1 = Rect {
//             position: Point { x: 10, y: 10 },
//             height: 100,
//             width: 100,
//         };

//         let rect2 = Rect {
//             position: Point { x: 200, y: 200 },
//             height: 100,
//             width: 100,
//         };

//         assert_eq!(rect2.intersects(&rect1), false);
//     }
// }


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
    fn update(&mut self);
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
        let mut game = game.initialize().await?;
        let mut game_loop = GameLoop {
            last_frame: browser::now()?,
            accumulated_delta: 0.0,
        };

        let renderer = Renderer {
            context: browser::context()?
        };
        
        let f: SharedLoopClosure = Rc::new(RefCell::new(None));
        let g = f.clone();

        *g.borrow_mut() = Some(browser::create_raf_closure(move |perf: f64| {
            game_loop.accumulated_delta += (perf - game_loop.last_frame) as f32;
            // game.drawに時間がかかると、updateが呼ばれる回数が減るため、その分を補填。描画を犠牲にして内部処理は確実に行うようにする。(drawを行わないupdateを行う)
            while game_loop.accumulated_delta > FRAME_SIZE {
                game.update();
                game_loop.accumulated_delta -= FRAME_SIZE;
            }
            game_loop.last_frame = perf;
            game.draw(&renderer);

            browser::request_animation_frame(f.borrow().as_ref().unwrap());
        }));
        browser::request_animation_frame(
            g.borrow().as_ref().ok_or_else(|| anyhow!("GameLoop: Loop is None"))?
        );
        Ok(())
    }
}

pub struct Renderer {
    context: CanvasRenderingContext2d,
}

impl Renderer {
    pub fn clear(&self, rect: &Rect) {
        self.context.clear_rect(rect.x.into(), rect.y.into(), rect.width.into(), rect.height.into());
    }
    pub fn draw_image(&self, image: &HtmlImageElement, frame: &Rect, destination: &Rect) {
        self.context.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
            image,
            frame.x.into(),
            frame.y.into(),
            frame.width.into(),
            frame.height.into(),
            destination.x.into(),
            destination.y.into(),
            destination.width.into(),
            destination.height.into(),
        ).expect("Drawing is thrown exceptions! Unrecoverable error.");
    }
}