use ggez::event::{self, KeyCode, KeyMods};
use ggez::*;
use specs::*;
use specs_derive::*;
use std::sync::Arc;

struct State {
    specs_world: World,
    player_input: Direction,
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

impl ggez::event::EventHandler for State {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);
        let positions = self.specs_world.read_storage::<Position>();
        let images = self.specs_world.read_storage::<Image>();

        for (p, i) in (&positions, &images).join() {
            graphics::draw(
                ctx,
                &*i.image,
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
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods) {
        if let KeyCode::Space = keycode {
            self.player_input.jump = false;
        }
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

    let char_image = match graphics::Image::new(ctx, "/player1.png") {
        Ok(img) => img,
        Err(e) => {
            panic!("Error: {}", e);
        }
    };
    let character = Arc::new(char_image);

    world
        .create_entity()
        .with(Position {
            position: nalgebra::Point2::new(100.0, 200.0),
        })
        .with(Image { image: character })
        .build();

    let player_input = Direction::new();
    let player_input_world = Direction::new();
    world.insert(player_input_world);

    let state = &mut State {
        specs_world: world,
        player_input,
    };

    event::run(ctx, event_loop, state).unwrap();
}
