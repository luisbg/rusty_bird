use ggez::event::{self, KeyCode, KeyMods};
use ggez::*;
use specs::*;
use specs_derive::*;
use std::path;
use std::sync::Arc;

const GRAVITY: f32 = 0.3;

#[derive(Default)]
pub struct Game {
    playing: bool,
}

impl Game {
    pub fn new() -> Self {
        Game { playing: true }
    }
}

struct State {
    specs_world: World,
    player_input: Direction,
    movement_system: MovementSystem,
    animation_system: AnimationSystem,
    collision_system: CollisionSystem,
}

#[derive(Component, Debug, PartialEq, Clone)]
#[storage(VecStorage)]
struct Image {
    image: Arc<graphics::Image>,
}

impl Image {
    pub fn new(ctx: &mut Context, path: &str) -> Self {
        let new_image = match graphics::Image::new(ctx, path) {
            Ok(img) => img,
            Err(e) => {
                panic!("Error: {}", e);
            }
        };

        Image {
            image: Arc::new(new_image),
        }
    }
}

#[derive(Component, Debug, PartialEq)]
#[storage(VecStorage)]
struct Position {
    position: nalgebra::Point2<f32>,
    speed: nalgebra::Point2<f32>,
}

#[derive(Clone, Copy, Default)]
struct Direction {
    jump: bool,
    release: bool,
}

impl Direction {
    fn new() -> Self {
        Direction {
            jump: false,
            release: true,
        }
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

#[derive(Component)]
#[storage(VecStorage)]
struct BackgroundTag {
    velocity: f32,
    width: f32,
    num_copies: u32,
}

#[derive(Component, Default)]
#[storage(NullStorage)]
struct ObstacleTag;

struct MovementSystem;
impl<'a> System<'a> for MovementSystem {
    type SystemData = (
        Write<'a, Direction>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Animation>,
        ReadStorage<'a, BackgroundTag>,
        ReadStorage<'a, ObstacleTag>,
        WriteStorage<'a, CollisionBox>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut dir, mut pos, anim, bg, obs, mut coll) = data;

        for (pos, _) in (&mut pos, &anim).join() {
            if dir.jump && dir.release {
                if pos.speed.y > -10.0 {
                    pos.speed.y -= 10.0;
                }
                dir.jump = false;
            } else if pos.speed.y < 6.0 {
                pos.speed.y += GRAVITY;
            }

            pos.position.y += pos.speed.y;

            if pos.position.y < 0.0 {
                pos.position.y = 0.0;
                pos.speed.y = 0.0;
            } else if pos.position.y > 460.0 {
                pos.position.y = 460.0;
                pos.speed.y = 0.0;
            }
        }

        for (pos, bg, _) in (&mut pos, &bg, !&obs).join() {
            pos.position.x -= bg.velocity;

            if pos.position.x < (bg.width * -1.0) {
                pos.position.x += bg.width * bg.num_copies as f32;
            }
        }

        for (pos, bg, _) in (&mut pos, &bg, &obs).join() {
            pos.position.x -= bg.velocity;

            if pos.position.x < (bg.width * -1.0) {
                pos.position.x = 1024.0;
            }
        }

        for (pos, coll_box) in (&mut pos, &mut coll).join() {
            // if an entity has an updated position, we also need to update it's collision box
            coll_box.origin.x = pos.position.x;
            coll_box.origin.y = pos.position.y;
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

#[derive(Component, Copy, Clone, Debug, PartialEq)]
#[storage(VecStorage)]
struct CollisionBox {
    origin: nalgebra::Point2<f32>,
    height: f32,
    width: f32,
}

struct CollisionSystem;

impl<'a> System<'a> for CollisionSystem {
    type SystemData = (
        ReadStorage<'a, Position>,
        ReadStorage<'a, CollisionBox>,
        ReadStorage<'a, Animation>,
        Write<'a, Game>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (pos, coll_box, anim, mut game) = data;

        let mut collided = false;
        // Find the player collision box
        for (player_box, _) in (&coll_box, &anim).join() {
            // Now check all entities with a collision box that aren't player controlled
            for (_, coll_box, _) in (&pos, &coll_box, !&anim).join() {
                if player_box.origin.x < coll_box.origin.x + coll_box.width
                    && player_box.origin.x + player_box.width > coll_box.origin.x
                    && player_box.origin.y < coll_box.origin.y + coll_box.height
                    && player_box.origin.y + player_box.height > coll_box.origin.y
                {
                    collided = true;
                }
            }
        }

        if collided {
            game.playing = false;
        }
    }
}

impl ggez::event::EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let game = self.specs_world.read_resource::<Game>();
        if !game.playing {
            return Ok(());
        }
        drop(game);

        const ANIMATION_DESIRED_FPS: u32 = 15;

        while timer::check_update_time(ctx, ANIMATION_DESIRED_FPS) {
            self.animation_system.run_now(&self.specs_world);
        }

        self.movement_system.run_now(&self.specs_world);
        self.collision_system.run_now(&self.specs_world);

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::Color::new(0.1, 0.1, 0.1, 1.0));
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
                    self.player_input.release = false;
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
            self.player_input.release = true;
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
    conf.window_mode.height = 600.0;
    conf.window_mode.width = 1024.0;

    let (ref mut ctx, ref mut event_loop) =
        ContextBuilder::new("rusty_bird", "Luis de Bethencourt")
            .conf(conf)
            .add_resource_path(path::PathBuf::from("./assets"))
            .build()
            .unwrap();

    let mut world = World::new();
    world.register::<Position>();
    world.register::<Image>();
    world.register::<Animation>();
    world.register::<BackgroundTag>();
    world.register::<ObstacleTag>();
    world.register::<CollisionBox>();

    // Background
    let bg_copies = 3;
    for level in 1..3 {
        let bg_image = Image::new(ctx, format!("/background{}.png", level).as_str());

        for n in 0..bg_copies {
            world
                .create_entity()
                .with(Position {
                    position: nalgebra::Point2::new(760.0 * n as f32, 0.0),
                    speed: nalgebra::Point2::new(0.0, 0.0),
                })
                .with(BackgroundTag {
                    velocity: 1.0 + level as f32,
                    width: 760.0,
                    num_copies: bg_copies,
                })
                .with(bg_image.clone())
                .build();
        }
    }

    // Floor
    let floor_image = Image::new(ctx, "/floor.png");
    let floor_copies = 5;
    for n in 0..floor_copies {
        world
            .create_entity()
            .with(Position {
                position: nalgebra::Point2::new(320.0 * n as f32, 520.0),
                speed: nalgebra::Point2::new(0.0, 0.0),
            })
            .with(BackgroundTag {
                velocity: 4.0,
                width: 320.0,
                num_copies: floor_copies,
            })
            .with(floor_image.clone())
            .build();
    }

    // Obstacle pipe
    let pipe_img = Image::new(ctx, "/bottom_pipe.png");
    for n in 0..2 {
        let pos_x = (500.0 * n as f32) + 900.0;
        let pos_y = 360.0;
        world
            .create_entity()
            .with(Position {
                position: nalgebra::Point2::new(pos_x, pos_y),
                speed: nalgebra::Point2::new(0.0, 0.0),
            })
            .with(pipe_img.clone())
            .with(BackgroundTag {
                velocity: 4.0,
                width: 64.0,
                num_copies: 1,
            })
            .with(ObstacleTag)
            .with(CollisionBox {
                origin: nalgebra::Point2::new(pos_x, pos_y),
                height: 240.0,
                width: 64.0,
            })
            .build();
    }

    // The bird
    let bird_height = 72.0;
    let bird_width = 61.0;
    world
        .create_entity()
        .with(Position {
            position: nalgebra::Point2::new(100.0, 200.0),
            speed: nalgebra::Point2::new(0.0, 0.0),
        })
        .with(Animation::from_frames(ctx, 4, "/player"))
        .with(CollisionBox {
            origin: nalgebra::Point2::new(100.0, 200.0),
            height: bird_height,
            width: bird_width,
        })
        .build();

    let game = Game::new();
    let player_input = Direction::new();
    let player_input_world = Direction::new();
    world.insert(player_input_world);
    world.insert(game);

    let update_pos = MovementSystem;
    let update_animation = AnimationSystem;
    let collision_system = CollisionSystem;

    let state = &mut State {
        specs_world: world,
        player_input,
        movement_system: update_pos,
        animation_system: update_animation,
        collision_system,
    };

    event::run(ctx, event_loop, state).unwrap();
}
