use create::engine::{Game, Renderer};
use anyhow::Result;
use async_trait::async_trait;


#[derive(Deserialize)]
struct SheetRect{
    x: i16,
    y: i16,
    w: i16,
    h: i16,
}

#[derive(Deserialize)]
struct Cell {
    frame: SheetRect,
}

#[derive(Deserialize)]
struct Sheet {
    frames: HashMap<String, Cell>,
}

pub struct WalkTheDog{
    image: HtmlImageElement,
    sheet: Sheet,
    frame: u8,
}

impl Game for WalkTheDog {
    async fn initialize(&self) -> Result<Box<dyn Game>> {
        Ok(Box::new(WalkTheDog{}))
    }
    fn update(&mut self) {
        // 何もしない
    }
    fn draw(&self, renderer: &Renderer) {
        let frame_name = format!("Run ({}).png", self.frame + 1);
        let sprite = self.sheet
            .frames
            .get(&frame_name)
            .expect("Cell not found in sheet");

        renderer.clear(Rect {
            x: 0.0,
            y: 0.0,
            w: 600.0,
            h: 600.0,
        });
        renderer.draw_image(&self.image, Rect {
            x: sprite.frame.x.into(),
            y: sprite.frame.y.into(),
            w: sprite.frame.w.into(),
            h: sprite.frame.h.into(),
        }, Rect {
            x: 300.0,
            y: 300.0,
            w: sprite.frame.w.into(),
            h: sprite.frame.h.into(),
        });
    }
}