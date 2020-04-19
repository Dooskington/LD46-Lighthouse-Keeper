use crate::game::*;
use specs::prelude::*;

pub struct StatsState {
    sanity: i32,
    food: i32,
    gas: i32,
    parts: i32,

    // TODO
    // flags list?
    // flags would be things like, GeneratorBroken, LensBroken, JunkAcquired,
}

impl StatsState {
    pub fn new() -> Self {
        StatsState {
            sanity: 10,
            food: 15,
            gas: 10,
            parts: 5,
        }
    }
}

#[derive(Default)]
pub struct StatsSystem {
    game_event_reader: Option<ReaderId<GameEvent>>,
}

impl<'a> System<'a> for StatsSystem {
    type SystemData = (
        ReadExpect<'a, EventChannel<GameEvent>>,
        WriteExpect<'a, StatsState>,
    );

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);

        self.game_event_reader = Some(
            world
                .fetch_mut::<EventChannel<GameEvent>>()
                .register_reader(),
        );
    }

    fn run(&mut self, (game_events, mut stats): Self::SystemData) {
        // TODO
        // intercept NewDayStarted events
        // every day, consume 1 food [x]
        // every 2 days, consume gasoline and flag generator as broken
        // every 4-7 days, merchant arrives

        for event in game_events.read(&mut self.game_event_reader.as_mut().unwrap()) {
            match event {
                GameEvent::NewDayStarted => {
                    stats.food -= 1;
                    if stats.food <= 0 {
                        // TODO
                    }

                    println!("1 food consumed.");
                },
                _ => {},
            }
        }
    }
}

#[derive(Default)]
pub struct StatsInfoRenderSystem;

impl<'a> System<'a> for StatsInfoRenderSystem {
    type SystemData = (Write<'a, RenderState>, ReadExpect<'a, StatsState>);

    fn run(&mut self, (mut render, stats): Self::SystemData) {
        // Sanity icon
        render.bind_transparency(Transparency::Opaque);
        render.bind_texture(resources::TEX_SPRITESHEET_UI);
        render.bind_color(COLOR_WHITE);
        render.bind_layer(layers::LAYER_UI);
        render.sprite(
            520.0,
            10.0,
            Point2f::new(0.0, 0.0),
            Vector2f::new(0.5, 0.5),
            SpriteRegion {
                x: 608,
                y: 0,
                w: 96,
                h: 96,
            },
        );

        // Food icon
        render.sprite(
            520.0,
            85.0,
            Point2f::new(0.0, 0.0),
            Vector2f::new(0.5, 0.5),
            SpriteRegion {
                x: 704,
                y: 0,
                w: 96,
                h: 96,
            },
        );

        // Parts icon
        render.sprite(
            520.0,
            150.0,
            Point2f::new(0.0, 0.0),
            Vector2f::new(0.5, 0.5),
            SpriteRegion {
                x: 800,
                y: 0,
                w: 96,
                h: 96,
            },
        );

        // Gas icon
        render.sprite(
            520.0,
            215.0,
            Point2f::new(0.0, 0.0),
            Vector2f::new(0.5, 0.5),
            SpriteRegion {
                x: 896,
                y: 0,
                w: 96,
                h: 96,
            },
        );

        // Sanity text
        render.bind_texture(resources::TEX_FONT);
        render.bind_color(COLOR_BLACK);
        render.text(
            535.0,
            64.0,
            8,
            16,
            1.0,
            &format!("{}", stats.sanity),
        );

        // Food text
        render.text(
            535.0,
            135.0,
            8,
            16,
            1.0,
            &format!("{}", stats.food),
        );

        // Parts text
        render.text(
            535.0,
            205.0,
            8,
            16,
            1.0,
            &format!("{}", stats.parts),
        );

        // Gas text
        render.text(
            535.0,
            270.0,
            8,
            16,
            1.0,
            &format!("{}", stats.gas),
        );
    }
}
