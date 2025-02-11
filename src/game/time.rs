use crate::game::*;
use specs::prelude::*;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum TimeOfDay {
    Morning,
    Afternoon,
    Night,
}

impl TimeOfDay {
    pub fn progress(&mut self) {
        if *self == TimeOfDay::Morning {
            *self = TimeOfDay::Afternoon;
        } else if *self == TimeOfDay::Afternoon {
            *self = TimeOfDay::Night;
        } else if *self == TimeOfDay::Night {
            *self = TimeOfDay::Morning;
        }
    }
}

impl std::fmt::Display for TimeOfDay {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let printable = match *self {
            TimeOfDay::Morning => "Morning",
            TimeOfDay::Afternoon => "Noon",
            TimeOfDay::Night => "Night",
            _ => "Unknown",
        };

        write!(f, "{}", printable)
    }
}

pub struct TimeState {
    pub day: i32,
    pub time_of_day: TimeOfDay,
    pub hours_passed: i32,
}

impl TimeState {
    pub fn new() -> Self {
        TimeState {
            day: 1,
            time_of_day: TimeOfDay::Morning,
            hours_passed: 0,
        }
    }
}

#[derive(Default)]
pub struct TimeSystem {
    game_event_reader: Option<ReaderId<GameEvent>>,
}

impl<'a> System<'a> for TimeSystem {
    type SystemData = (
        WriteExpect<'a, EventChannel<GameEvent>>,
        WriteExpect<'a, EventChannel<LogEvent>>,
        WriteExpect<'a, TimeState>,
    );

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);

        self.game_event_reader = Some(
            world
                .fetch_mut::<EventChannel<GameEvent>>()
                .register_reader(),
        );
    }

    fn run(&mut self, (mut game_events, mut log_events, mut time): Self::SystemData) {
        // NOTE (declan, 4/18/20)
        // what am I thinking
        let mut did_new_day_start = false;
        let mut did_new_time_of_day_start = false;

        for event in game_events
            .read(&mut self.game_event_reader.as_mut().unwrap())
            .cloned()
        {
            match event {
                GameEvent::ProgressTime { hours } => {
                    time.hours_passed += hours;
                    if time.hours_passed >= 4 {
                        time.hours_passed -= 4;
                        time.time_of_day.progress();
                        did_new_time_of_day_start = true;

                        if time.time_of_day == TimeOfDay::Morning {
                            did_new_day_start = true;
                            time.day += 1;
                        }
                    }
                }
                _ => {}
            }
        }

        if did_new_day_start {
            println!("A new day begins.");
            game_events.single_write(GameEvent::NewDayStarted { day: time.day });
        }

        if did_new_time_of_day_start {
            let msg = match time.time_of_day {
                TimeOfDay::Morning => "The cool sea air wakes you from slumber.",
                TimeOfDay::Afternoon => "The sun rises high in the sky.",
                TimeOfDay::Night => "The darkness of night creeps upon your lonely isle.",
            }.to_owned();

            log_events.single_write(LogEvent { message: msg, color: COLOR_YELLOW });

            game_events.single_write(GameEvent::NewTimeOfDayStarted {
                time_of_day: time.time_of_day,
            });
        }
    }
}

#[derive(Default)]
pub struct TimeInfoRenderSystem;

impl<'a> System<'a> for TimeInfoRenderSystem {
    type SystemData = (Write<'a, RenderState>, ReadExpect<'a, TimeState>);

    fn run(&mut self, (mut render, time): Self::SystemData) {
        // Time UI background
        render.bind_transparency(Transparency::Opaque);
        render.bind_texture(resources::TEX_SPRITESHEET_UI);
        render.bind_color(COLOR_WHITE);
        render.bind_layer(layers::LAYER_UI);
        render.sprite(
            0.0,
            0.0,
            Point2f::origin(),
            Vector2f::new(0.5, 0.5),
            SpriteRegion {
                x: 0,
                y: 0,
                w: 320,
                h: 160,
            },
        );

        let hours_bar_sprite_region = match time.hours_passed {
            0 => SpriteRegion {
                x: 320,
                y: 0,
                w: 288,
                h: 64,
            },
            1 => SpriteRegion {
                x: 320,
                y: 64,
                w: 288,
                h: 64,
            },
            2 => SpriteRegion {
                x: 320,
                y: 128,
                w: 288,
                h: 64,
            },
            3 => SpriteRegion {
                x: 320,
                y: 192,
                w: 288,
                h: 64,
            },
            _ => SpriteRegion {
                x: 320,
                y: 256,
                w: 288,
                h: 64,
            },
        };

        render.sprite(
            0.0,
            73.0,
            Point2f::origin(),
            Vector2f::new(0.5, 0.5),
            hours_bar_sprite_region,
        );

        let time_of_day_sprite_region = match time.time_of_day {
            TimeOfDay::Morning => SpriteRegion {
                x: 0,
                y: 256,
                w: 64,
                h: 64,
            },
            TimeOfDay::Afternoon => SpriteRegion {
                x: 64,
                y: 256,
                w: 64,
                h: 64,
            },
            _ => SpriteRegion {
                x: 128,
                y: 256,
                w: 64,
                h: 64,
            },
        };

        render.sprite(
            100.0,
            45.0,
            Point2f::new(0.0, 0.0),
            Vector2f::new(0.5, 0.5),
            time_of_day_sprite_region,
        );

        // Day Text
        render.bind_layer(layers::LAYER_UI);
        render.bind_transparency(Transparency::Opaque);
        render.bind_texture(resources::TEX_FONT);
        render.bind_color(COLOR_BLACK);
        render.text(8.0, 8.0, 8, 16, 2.0, &format!("Day {}", time.day));

        render.text(8.0, 48.0, 8, 16, 1.5, &format!("{}", time.time_of_day));
    }
}
