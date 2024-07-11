use self::red_hat_boy_states::*;
use crate::{
    browser,
    engine::{self, Cell, Game, Image, KeyState, Point, Rect, Renderer, Sheet},
};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use gloo_utils::format::JsValueSerdeExt;
use web_sys::HtmlImageElement;

const HEIGHT: i16 = 600;
const LOW_PLATFORM: i16 = 420;
const FIRST_PLATFORM: i16 = 370;
const HIGH_PLATFORM: i16 = 375;
pub enum WalkTheDog {
    Loading,
    Loaded(Walk),
}

impl WalkTheDog {
    pub fn new() -> Self {
        WalkTheDog::Loading
    }
}

pub struct Walk {
    boy: RedHatBoy,
    backgrounds: [Image; 2],
    stone: Image,
    platform: Platform,
}

impl Walk {
    fn velocity(&self) -> i16 {
        -self.boy.walking_speed()
    }
}

#[async_trait(?Send)]
impl Game for WalkTheDog {
    async fn initialize(&self) -> Result<Box<dyn Game>> {
        match self {
            WalkTheDog::Loading => {
                let json = browser::fetch_json("rhb.json").await?;
                let background = engine::load_image("BG.png").await?;
                let stone = engine::load_image("Stone.png").await?;
                let platform_sheet = browser::fetch_json("tiles.json").await?;
                let platform = Platform::new(
                    platform_sheet.into_serde::<Sheet>()?,
                    engine::load_image("tiles.png").await?,
                    Point {
                        x: FIRST_PLATFORM,
                        y: LOW_PLATFORM,
                    },
                );
                let rhb = RedHatBoy::new(
                    json.into_serde::<Sheet>()?,
                    engine::load_image("rhb.png").await?,
                );
                let background_width = background.width() as i16;
                Ok(Box::new(WalkTheDog::Loaded(Walk {
                    boy: rhb,
                    backgrounds: [
                        Image::new(background.clone(), Point { x: 0, y: 0 }),
                        Image::new(
                            background,
                            Point {
                                x: background_width,
                                y: 0,
                            },
                        ),
                    ],
                    stone: Image::new(stone, Point { x: 250, y: 546 }),
                    platform,
                })))
            }
            WalkTheDog::Loaded(_) => Err(anyhow!("Game already initialized")),
        }
    }
    fn update(&mut self, keystate: &KeyState) {
        if let WalkTheDog::Loaded(walk) = self {
            // 同時押しに対応するため、elseを使わずifで分岐している
            if keystate.is_pressed("ArrowRight") {
                walk.boy.run_right();
            }
            if keystate.is_pressed("ArrowDown") {
                walk.boy.slide();
            }
            if keystate.is_pressed("Space") {
                walk.boy.jump();
            }

            walk.boy.update();

            // boy以外のすべてのオブジェクトを動かす
            walk.platform.position.x += walk.velocity();
            walk.stone.move_horizonatally(walk.velocity());

            let velocity = walk.velocity();
            let [first_background, second_background] = &mut walk.backgrounds;
            first_background.move_horizonatally(velocity);
            second_background.move_horizonatally(velocity);

            if first_background.right() < 0 {
                first_background.set_x(second_background.right());
            }
            if second_background.right() < 0 {
                second_background.set_x(first_background.right());
            }

            for bounding_box in &walk.platform.bounding_boxes() {
                if walk.boy.bounding_box().intersects(bounding_box) {
                    if walk.boy.velocity_y() > 0 && walk.boy.pos_y() < walk.platform.position.y {
                        walk.boy.land_on(bounding_box.position.y);
                    } else {
                        walk.boy.knock_out();
                    }
                }
            }
            if walk
                .boy
                .bounding_box()
                .intersects(&walk.stone.bounding_box())
            {
                walk.boy.knock_out();
            }
        }
    }
    fn draw(&self, renderer: &Renderer) {
        renderer.clear(&Rect {
            position: Point { x: 0, y: 0 },
            width: HEIGHT,
            height: 600,
        });
        if let WalkTheDog::Loaded(walk) = self {
            walk.backgrounds.iter().for_each(|backgdound| {
                backgdound.draw(renderer);
            });
            walk.boy.draw(renderer);
            walk.stone.draw(renderer);
            walk.platform.draw(renderer);
        }
    }
}

pub struct RedHatBoy {
    state_machine: RedHatBoyStateMachine,
    sprite_sheet: Sheet,
    image: HtmlImageElement,
}
impl RedHatBoy {
    fn new(sheet: Sheet, image: HtmlImageElement) -> Self {
        RedHatBoy {
            state_machine: RedHatBoyStateMachine::Idle(RedHatBoyState::new()),
            sprite_sheet: sheet,
            image,
        }
    }
    fn frame_name(&self) -> String {
        format!(
            "{} ({}).png",
            self.state_machine.frame_name(),
            (self.state_machine.context().frame / 3) + 1
        )
    }
    fn current_sprite(&self) -> Option<&Cell> {
        self.sprite_sheet.frames.get(&self.frame_name())
    }
    fn destination_box(&self) -> Rect {
        let sprite = self.current_sprite().expect("Cell not found");
        Rect::new_from_x_y(
            (self.state_machine.context().position.x + sprite.sprite_source_size.x).into(),
            (self.state_machine.context().position.y + sprite.sprite_source_size.y).into(),
            sprite.frame.w.into(),
            sprite.frame.h.into(),
        )
    }
    fn bounding_box(&self) -> Rect {
        const X_OFFSET: i16 = 18;
        const Y_OFFSET: i16 = 14;
        const WIDTH_OFFFSET: i16 = 28;
        let mut bounding_box = self.destination_box();
        bounding_box.position.x += X_OFFSET;
        bounding_box.position.y += Y_OFFSET;
        bounding_box.width -= WIDTH_OFFFSET;
        bounding_box.height -= Y_OFFSET;
        bounding_box
    }

    fn draw(&self, renderer: &Renderer) {
        let sprite = self.current_sprite().expect("Cell not found");

        renderer.draw_image(
            &self.image,
            &&Rect::new_from_x_y(
                sprite.frame.x.into(),
                sprite.frame.y.into(),
                sprite.frame.w.into(),
                sprite.frame.h.into(),
            ),
            &self.destination_box(),
        );
    }
    fn update(&mut self) {
        self.state_machine = self.state_machine.update();
    }
    fn run_right(&mut self) {
        self.state_machine = self.state_machine.transition(Event::Run);
    }
    fn slide(&mut self) {
        self.state_machine = self.state_machine.transition(Event::Slide);
    }
    fn jump(&mut self) {
        self.state_machine = self.state_machine.transition(Event::Jump);
    }
    fn knock_out(&mut self) {
        self.state_machine = self.state_machine.transition(Event::KnockOut);
    }
    fn land_on(&mut self, position: i16) {
        self.state_machine = self.state_machine.transition(Event::Land(position));
    }
    fn pos_y(&self) -> i16 {
        self.state_machine.context().position.y
    }
    fn velocity_y(&self) -> i16 {
        self.state_machine.context().velocity.y
    }
    fn walking_speed(&self) -> i16 {
        self.state_machine.context().velocity.x
    }
}

mod red_hat_boy_states {
    use super::HEIGHT;
    use crate::engine::Point;
    const FLOOR: i16 = 479;
    const PLAYER_HEIGHT: i16 = HEIGHT - FLOOR;
    const STARTING_POINT: i16 = -20;
    const IDLE_FRAMES: u8 = 29;
    const IDLE_FRAME_NAME: &str = "Idle";
    const RUN_FRAME_NAME: &str = "Run";
    const RUNNING_FRAMES: u8 = 23;
    const RUNNING_SPEED: i16 = 4;
    const SLIDE_FRAME_NAME: &str = "Slide";
    const SLIDING_FRAMES: u8 = 14;
    const JUMP_FRAME_NAME: &str = "Jump";
    const JUMPING_FRAMES: u8 = 35; // 12(画像の枚数) * 3 - 1
    const JUMP_SPEED: i16 = -25;
    const GRAVITY: i16 = 1;
    const FALLING_FRAMES: u8 = 29;
    const FALLING_FRAME_NAME: &str = "Dead";
    const TERMINAL_VELOCITY: i16 = 20;

    #[derive(Copy, Clone)]
    pub struct RedHatBoyState<S> {
        context: RedHatBoyContext,
        _state: S,
    }

    impl<S> RedHatBoyState<S> {
        pub fn context(&self) -> &RedHatBoyContext {
            &self.context
        }
        fn update_context(&mut self, frames: u8) {
            self.context = self.context.clone().update(frames);
        }
    }

    #[derive(Copy, Clone)]
    pub struct RedHatBoyContext {
        pub frame: u8,
        pub position: Point,
        pub velocity: Point,
    }

    impl RedHatBoyContext {
        pub fn update(mut self, frame_count: u8) -> Self {
            if self.velocity.y < TERMINAL_VELOCITY {
                self.velocity.y += GRAVITY;
            }
            if self.frame < frame_count {
                self.frame += 1;
            } else {
                self.frame = 0;
            }
            self.position.y += self.velocity.y;

            if self.position.y > FLOOR {
                self.position.y = FLOOR;
            }
            self
        }
        fn reset_frame(mut self) -> Self {
            self.frame = 0;
            self
        }
        fn run_right(mut self) -> Self {
            self.velocity.x = RUNNING_SPEED;
            self
        }
        fn set_vertical_velocity(mut self, speed: i16) -> Self {
            self.velocity.y = speed;
            self
        }
        fn stop(mut self) -> Self {
            self.velocity.x = 0;
            self.velocity.y = 0;
            self
        }
        fn set_on(mut self, position: i16) -> Self {
            let position = position - PLAYER_HEIGHT;
            self.position.y = position;
            self
        }
    }

    #[derive(Copy, Clone)]
    pub struct Idle;
    #[derive(Copy, Clone)]
    pub struct Running;
    #[derive(Copy, Clone)]
    pub struct Sliding;

    #[derive(Copy, Clone)]
    pub struct Jumping;
    #[derive(Copy, Clone)]
    pub struct Falling;

    #[derive(Copy, Clone)]
    pub struct KnockedOut;

    impl RedHatBoyState<Idle> {
        pub fn new() -> Self {
            RedHatBoyState {
                context: RedHatBoyContext {
                    frame: 0,
                    position: Point {
                        x: STARTING_POINT,
                        y: FLOOR,
                    },
                    velocity: Point { x: 0, y: 0 },
                },
                _state: Idle {},
            }
        }
        pub fn run(self) -> RedHatBoyState<Running> {
            RedHatBoyState {
                context: self.context.reset_frame().run_right(),
                _state: Running {},
            }
        }
        pub fn frame_name(&self) -> &str {
            IDLE_FRAME_NAME
        }
        pub fn update(mut self) -> Self {
            self.context = self.context.update(IDLE_FRAMES);
            self
        }
    }

    impl RedHatBoyState<Running> {
        pub fn frame_name(&self) -> &str {
            RUN_FRAME_NAME
        }
        pub fn update(mut self) -> Self {
            self.context = self.context.update(RUNNING_FRAMES);
            self
        }
        pub fn slide(self) -> RedHatBoyState<Sliding> {
            RedHatBoyState {
                context: self.context.reset_frame(),
                _state: Sliding {},
            }
        }

        pub fn jump(self) -> RedHatBoyState<Jumping> {
            RedHatBoyState {
                context: self.context.set_vertical_velocity(JUMP_SPEED).reset_frame(),
                _state: Jumping {},
            }
        }
        pub fn knock_out(self) -> RedHatBoyState<Falling> {
            RedHatBoyState {
                context: self.context.reset_frame().stop(),
                _state: Falling {},
            }
        }
        pub fn land_on(self, position: i16) -> RedHatBoyState<Running> {
            RedHatBoyState {
                context: self.context.set_on(position),
                _state: Running {},
            }
        }
    }
    impl RedHatBoyState<Sliding> {
        pub fn frame_name(&self) -> &str {
            SLIDE_FRAME_NAME
        }
        pub fn update(mut self) -> SlidingEndState {
            self.context = self.context.update(SLIDING_FRAMES);

            if self.context.frame >= SLIDING_FRAMES {
                SlidingEndState::Complete(self.stand())
            } else {
                SlidingEndState::Sliding(self)
            }
        }
        pub fn stand(self) -> RedHatBoyState<Running> {
            RedHatBoyState {
                context: self.context.reset_frame(),
                _state: Running {},
            }
        }
        pub fn knock_out(self) -> RedHatBoyState<Falling> {
            RedHatBoyState {
                context: self.context.reset_frame().stop(),
                _state: Falling {},
            }
        }
        pub fn land_on(self, position: i16) -> RedHatBoyState<Sliding> {
            RedHatBoyState {
                context: self.context.set_on(position),
                _state: Sliding {},
            }
        }
    }
    impl RedHatBoyState<Jumping> {
        pub fn frame_name(&self) -> &str {
            JUMP_FRAME_NAME
        }
        pub fn update(mut self) -> JumpingEndState {
            self.context = self.context.update(JUMPING_FRAMES);

            if self.context.position.y >= FLOOR {
                JumpingEndState::Landing(self.land_on(HEIGHT.into()))
            } else {
                JumpingEndState::Jumping(self)
            }
        }
        pub fn land_on(self, position: i16) -> RedHatBoyState<Running> {
            RedHatBoyState {
                context: self.context.reset_frame().set_on(position),
                _state: Running {},
            }
        }
        pub fn knock_out(self) -> RedHatBoyState<Falling> {
            RedHatBoyState {
                context: self.context.reset_frame().stop(),
                _state: Falling {},
            }
        }
    }
    impl RedHatBoyState<Falling> {
        pub fn frame_name(&self) -> &str {
            FALLING_FRAME_NAME
        }

        pub fn knock_out(self) -> RedHatBoyState<KnockedOut> {
            RedHatBoyState {
                context: self.context,
                _state: KnockedOut {},
            }
        }

        pub fn update(mut self) -> FallingEndState {
            self.update_context(FALLING_FRAMES);
            if self.context.frame >= FALLING_FRAMES {
                FallingEndState::KnockedOut(self.knock_out())
            } else {
                FallingEndState::Falling(self)
            }
        }
    }
    impl RedHatBoyState<KnockedOut> {
        pub fn frame_name(&self) -> &str {
            FALLING_FRAME_NAME
        }
    }
    pub enum SlidingEndState {
        Complete(RedHatBoyState<Running>),
        Sliding(RedHatBoyState<Sliding>),
    }
    pub enum JumpingEndState {
        Landing(RedHatBoyState<Running>),
        Jumping(RedHatBoyState<Jumping>),
    }
    pub enum FallingEndState {
        KnockedOut(RedHatBoyState<KnockedOut>),
        Falling(RedHatBoyState<Falling>),
    }
}

pub enum Event {
    Run,
    Slide,
    Jump,
    KnockOut,
    Update,
    Land(i16),
}
#[derive(Copy, Clone)]
enum RedHatBoyStateMachine {
    Idle(RedHatBoyState<Idle>),
    Running(RedHatBoyState<Running>),
    Sliding(RedHatBoyState<Sliding>),
    Jumping(RedHatBoyState<Jumping>),
    Falling(RedHatBoyState<Falling>),
    KnockedOut(RedHatBoyState<KnockedOut>),
}
impl RedHatBoyStateMachine {
    fn transition(self, event: Event) -> Self {
        match (self, event) {
            (RedHatBoyStateMachine::Idle(state), Event::Run) => state.run().into(),
            (RedHatBoyStateMachine::Running(state), Event::Slide) => state.slide().into(),
            (RedHatBoyStateMachine::Idle(state), Event::Update) => state.update().into(),
            (RedHatBoyStateMachine::Running(state), Event::Update) => state.update().into(),
            (RedHatBoyStateMachine::Sliding(state), Event::Update) => state.update().into(),
            (RedHatBoyStateMachine::Running(state), Event::Jump) => state.jump().into(),
            (RedHatBoyStateMachine::Jumping(state), Event::Update) => state.update().into(),
            (RedHatBoyStateMachine::Sliding(state), Event::KnockOut) => state.knock_out().into(),
            (RedHatBoyStateMachine::Jumping(state), Event::KnockOut) => state.knock_out().into(),
            (RedHatBoyStateMachine::Running(state), Event::KnockOut) => state.knock_out().into(),
            (RedHatBoyStateMachine::Falling(state), Event::Update) => state.update().into(),
            (RedHatBoyStateMachine::Jumping(state), Event::Land(position)) => {
                state.land_on(position).into()
            }
            (RedHatBoyStateMachine::Running(state), Event::Land(position)) => {
                state.land_on(position).into()
            }
            (RedHatBoyStateMachine::Sliding(state), Event::Land(position)) => {
                state.land_on(position).into()
            }
            _ => self,
        }
    }
    fn frame_name(&self) -> &str {
        match self {
            RedHatBoyStateMachine::Idle(state) => state.frame_name(),
            RedHatBoyStateMachine::Running(state) => state.frame_name(),
            RedHatBoyStateMachine::Sliding(state) => state.frame_name(),
            RedHatBoyStateMachine::Jumping(state) => state.frame_name(),
            RedHatBoyStateMachine::Falling(state) => state.frame_name(),
            RedHatBoyStateMachine::KnockedOut(state) => state.frame_name(),
        }
    }
    fn context(&self) -> &RedHatBoyContext {
        match self {
            RedHatBoyStateMachine::Idle(state) => &state.context(),
            RedHatBoyStateMachine::Running(state) => &state.context(),
            RedHatBoyStateMachine::Sliding(state) => &state.context(),
            RedHatBoyStateMachine::Jumping(state) => &state.context(),
            RedHatBoyStateMachine::Falling(state) => &state.context(),
            RedHatBoyStateMachine::KnockedOut(state) => &state.context(),
        }
    }

    fn update(self) -> Self {
        self.transition(Event::Update)
    }
}
impl From<RedHatBoyState<Running>> for RedHatBoyStateMachine {
    fn from(state: RedHatBoyState<Running>) -> Self {
        RedHatBoyStateMachine::Running(state)
    }
}

impl From<RedHatBoyState<Idle>> for RedHatBoyStateMachine {
    fn from(state: RedHatBoyState<Idle>) -> Self {
        RedHatBoyStateMachine::Idle(state)
    }
}
impl From<RedHatBoyState<Sliding>> for RedHatBoyStateMachine {
    fn from(state: RedHatBoyState<Sliding>) -> Self {
        RedHatBoyStateMachine::Sliding(state)
    }
}

impl From<RedHatBoyState<Jumping>> for RedHatBoyStateMachine {
    fn from(state: RedHatBoyState<Jumping>) -> Self {
        RedHatBoyStateMachine::Jumping(state)
    }
}

impl From<RedHatBoyState<Falling>> for RedHatBoyStateMachine {
    fn from(state: RedHatBoyState<Falling>) -> Self {
        RedHatBoyStateMachine::Falling(state)
    }
}
impl From<RedHatBoyState<KnockedOut>> for RedHatBoyStateMachine {
    fn from(state: RedHatBoyState<KnockedOut>) -> Self {
        RedHatBoyStateMachine::KnockedOut(state)
    }
}

impl From<SlidingEndState> for RedHatBoyStateMachine {
    fn from(end_state: SlidingEndState) -> Self {
        match end_state {
            SlidingEndState::Complete(running_state) => running_state.into(),
            SlidingEndState::Sliding(sliding_state) => sliding_state.into(),
        }
    }
}

impl From<JumpingEndState> for RedHatBoyStateMachine {
    fn from(end_state: JumpingEndState) -> Self {
        match end_state {
            JumpingEndState::Landing(running_state) => running_state.into(),
            JumpingEndState::Jumping(jumping_state) => jumping_state.into(),
        }
    }
}
impl From<FallingEndState> for RedHatBoyStateMachine {
    fn from(state: FallingEndState) -> Self {
        match state {
            FallingEndState::Falling(falling) => falling.into(),
            FallingEndState::KnockedOut(knocked_out) => knocked_out.into(),
        }
    }
}

struct Platform {
    sheet: Sheet,
    image: HtmlImageElement,
    position: Point,
}

impl Platform {
    fn new(sheet: Sheet, image: HtmlImageElement, position: Point) -> Self {
        Platform {
            sheet,
            image,
            position,
        }
    }
    fn draw(&self, renderer: &Renderer) {
        let platform = self.sheet.frames.get("13.png").expect("Cell not found");
        renderer.draw_image(
            &self.image,
            &Rect::new_from_x_y(
                platform.frame.x.into(),
                platform.frame.y.into(),
                (platform.frame.w * 3).into(),
                platform.frame.h.into(),
            ),
            &Rect::new_from_x_y(
                self.position.x.into(),
                self.position.y.into(),
                (platform.frame.w * 3).into(),
                platform.frame.h.into(),
            ),
        );
    }
    fn destination_box(&self) -> Rect {
        let platform = self.sheet.frames.get("13.png").expect("Cell not found");
        Rect::new_from_x_y(
            self.position.x.into(),
            self.position.y.into(),
            (platform.frame.w * 3).into(),
            platform.frame.h.into(),
        )
    }
    fn bounding_boxes(&self) -> Vec<Rect> {
        const X_OFFSET: i16 = 60;
        const END_HEIGHT: i16 = 54;
        let destination_box = self.destination_box();
        let bounding_box_one = Rect::new_from_x_y(
            destination_box.position.x,
            destination_box.position.y,
            X_OFFSET,
            END_HEIGHT,
        );
        let bounding_box_two = Rect::new_from_x_y(
            destination_box.position.x + X_OFFSET,
            destination_box.position.y,
            destination_box.width - (X_OFFSET * 2),
            destination_box.height,
        );
        let bounding_box_three = Rect::new_from_x_y(
            destination_box.position.x + destination_box.width - X_OFFSET,
            destination_box.position.y,
            X_OFFSET,
            END_HEIGHT,
        );

        vec![bounding_box_one, bounding_box_two, bounding_box_three]
    }
}
