use ggez::event::{self, KeyCode, KeyMods};
use ggez::*;
use specs::*;
use specs_derive::*;
use std::sync::Arc;

struct State {
    specs_world: World,
    player_input: Direction,
    movement_system: MovementSystem,
    animation_system: AnimationSystem,
}

#[derive(Component, Debug, PartialEq)]
#[storage(VecStorage)]
struct Image {
    image: Arc<graphics::Image>,
}

#[derive(Component, Debug, PartialEq)]
#[storage(VecStorage)]
struct Position {
    position: nalgebra::Point2<f32>,
}

#[derive(Clone, Copy, Default)]
struct Direction {
    jump: bool,
}

impl Direction {
    fn new() -> Self {
        Direction { jump: false }
    }
}

#[derive(Component, Default, Debug)]
#[storage(VecStorage)]
struct Animation {
    pub current_frame: u32,
    max: u32,
    pub images: Vec<graphics::Image>,
}

impl Animation {
    fn new(max: u32, images: Vec<graphics::Image>) -> Self {
        Animation {
            current_frame: 0,
            max,
            images,
        }
    }

    fn from_frames(ctx: &mut Context, frames: u32, base_path: &str) -> Self {
        let mut character_anim = Vec::new();

        for n in 1..frames + 1 {
            let path = format!("{}{}.png", base_path, n);
            character_anim.push(graphics::Image::new(ctx, path).unwrap());
        }

        Animation::new(frames, character_anim)
    }
}

struct MovementSystem;
impl<'a> System<'a> for MovementSystem {
    type SystemData = (Read<'a, Direction>, WriteStorage<'a, Position>);

    fn run(&mut self, data: Self::SystemData) {
        let (dir, mut pos) = data;

        for pos in (&mut pos).join() {
            if dir.jump {
                pos.position.y -= 10.0;
            }
        }
    }
}

struct AnimationSystem;
impl<'a> System<'a> for AnimationSystem {
    type SystemData = (WriteStorage<'a, Animation>, ReadStorage<'a, Image>);

    fn run(&mut self, data: Self::SystemData) {
        let (mut anim, _img) = data;

        for anim in (&mut anim).join() {
            anim.current_frame += 1;
            if anim.current_frame >= anim.max {
                anim.current_frame = 0;
            }
        }
    }
}

impl ggez::event::EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        const ANIMATION_DESIRED_FPS: u32 = 15;

        while timer::check_update_time(ctx, ANIMATION_DESIRED_FPS) {
            self.animation_system.run_now(&self.specs_world);
        }

        self.movement_system.run_now(&self.specs_world);

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);
        let positions = self.specs_world.read_storage::<Position>();
        let images = self.specs_world.read_storage::<Image>();
        let animations = self.specs_world.read_storage::<Animation>();

        for (p, i) in (&positions, &images).join() {
            graphics::draw(
                ctx,
                &*i.image,
                graphics::DrawParam::default().dest(p.position),
            )
            .unwrap_or_else(|err| println!("draw error {:?}", err));
        }

        for (p, a) in (&positions, &animations).join() {
            graphics::draw(
                ctx,
                &(*a).images[(*a).current_frame as usize].clone(),
                graphics::DrawParam::default().dest(p.position),
            )
            .unwrap_or_else(|err| println!("draw error {:?}", err));
        }

        graphics::present(ctx)?;

        timer::yield_now();
        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        _keymod: KeyMods,
        repeat: bool,
    ) {
        if !repeat {
            match keycode {
                KeyCode::Space => {
                    self.player_input.jump = true;
                }
                KeyCode::Escape => {
                    event::quit(ctx);
                }
                _ => (),
            }
        }

        let mut input_state = self.specs_world.write_resource::<Direction>();
        *input_state = self.player_input;
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods) {
        if let KeyCode::Space = keycode {
            self.player_input.jump = false;
        }

        let mut input_state = self.specs_world.write_resource::<Direction>();
        *input_state = self.player_input;
    }
}

fn main() {
    println!("Rusty Bird");

    let mut conf = conf::Conf::new();
    let win_setup = conf::WindowSetup {
        title: "Rusty Bird".to_owned(),
        samples: conf::NumSamples::Zero,
        vsync: true,
        icon: "".to_owned(),
        srgb: true,
    };
    conf.window_setup = win_setup;
    conf.window_mode.height = 720.0;
    conf.window_mode.width = 1280.0;

    let (ref mut ctx, ref mut event_loop) =
        ContextBuilder::new("rusty_bird", "Luis de Bethencourt")
            .conf(conf)
            .build()
            .unwrap();

    let mut world = World::new();
    world.register::<Position>();
    world.register::<Image>();
    world.register::<Animation>();

    // The bird
    world
        .create_entity()
        .with(Position {
            position: nalgebra::Point2::new(100.0, 200.0),
        })
        .with(Animation::from_frames(ctx, 4, "/player"))
        .build();

    let player_input = Direction::new();
    let player_input_world = Direction::new();
    world.insert(player_input_world);

    let update_pos = MovementSystem;
    let update_animation = AnimationSystem;

    let state = &mut State {
        specs_world: world,
        player_input,
        movement_system: update_pos,
        animation_system: update_animation,
    };

    event::run(ctx, event_loop, state).unwrap();
}
