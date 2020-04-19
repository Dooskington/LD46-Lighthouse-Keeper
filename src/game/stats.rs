use crate::game::*;
use specs::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Copy)]
pub enum StatEffect {
    Add { stat: Stat, amount: i32 },
    Subtract { stat: Stat, amount: i32 },
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub enum Stat {
    Sanity,
    Food,
    Gas,
    Parts,
    Money,
}

impl std::fmt::Display for Stat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let printable = match *self {
            Stat::Sanity => "Sanity",
            Stat::Food => "Food",
            Stat::Gas => "Gas",
            Stat::Parts => "Parts",
            Stat::Money => "Dollars",
            _ => "Unknown",
        };

        write!(f, "{}", printable)
    }
}

pub struct StatsState {
    stats: HashMap<Stat, i32>,

    // TODO
    // flags list?
    // flags would be things like, GeneratorBroken, LensBroken, JunkAcquired,
}

impl StatsState {
    pub fn new() -> Self {
        let mut stats = HashMap::new();
        stats.insert(Stat::Sanity, 10);
        stats.insert(Stat::Food, 8);
        stats.insert(Stat::Gas, 8);
        stats.insert(Stat::Parts, 5);
        stats.insert(Stat::Money, 20);

        StatsState {
            stats,
        }
    }

    pub fn stat(&self, stat: Stat) -> i32 {
        self.stats.get(&stat).unwrap_or(&0).clone()
    }

    pub fn add(&mut self, stat: Stat, amount: i32) {
        let entry = self.stats.entry(stat).or_insert(0);
        *entry += amount;
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
                    stats.add(Stat::Food, -1);
                    if stats.stat(Stat::Food) <= 0 {
                        // TODO
                    }

                    println!("You unpack the days rations from food storage. (Food -1)");
                },
                GameEvent::HandleStatEffects { effects } => {
                    for effect in effects {
                        match effect {
                            StatEffect::Add { stat, amount } => {
                                stats.add(*stat, *amount);
                                println!("({} +{})", stat, amount.abs());
                            }
                            StatEffect::Subtract { stat, amount } => {
                                stats.add(*stat, -*amount);
                                println!("({} -{})", stat, amount.abs());
                            }
                        }
                    }
                }
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
            &format!("{}", stats.stat(Stat::Sanity)),
        );

        // Food text
        render.text(
            535.0,
            135.0,
            8,
            16,
            1.0,
            &format!("{}", stats.stat(Stat::Food)),
        );

        // Parts text
        render.text(
            535.0,
            205.0,
            8,
            16,
            1.0,
            &format!("{}", stats.stat(Stat::Parts)),
        );

        // Gas text
        render.text(
            535.0,
            270.0,
            8,
            16,
            1.0,
            &format!("{}", stats.stat(Stat::Gas)),
        );

        // Money text
        render.text(
            160.0,
            8.0,
            8,
            16,
            1.5,
            &format!("${}", stats.stat(Stat::Money)),
        );
    }
}
