use ggez::*;

struct State {}

impl ggez::event::EventHandler for State {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }
}

fn main() {
    println!("Rusty Bird");
    let state = &mut State {};

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

    let (ref mut ctx, ref mut event_loop) = ContextBuilder::new("rusty_bird", "Luis de Bethencourt")
        .conf(conf)
        .build()
        .unwrap();

    event::run(ctx, event_loop, state).unwrap();
}
