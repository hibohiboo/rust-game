#[derive(Clone, Copy, Default)]
pub struct Point {
    pub x: i16,
    pub y: i16,
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
        self.x() < rect.right()
            && self.right() > rect.x()
            && self.y() < rect.bottom()
            && self.bottom() > rect.y()
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
mod tests { // 他のコードから隔離するため mod キーワードでモジュール化する
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
            position: Point { x: 100, y: 10 },
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
