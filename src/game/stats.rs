use crate::game::*;
use specs::prelude::*;
use std::collections::HashMap;
use rand::Rng;

#[derive(Clone, Copy, Debug)]
pub enum ConditionEffect {
    Set { condition: GameCondition },
    Clear { condition: GameCondition },
}

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
    conditions: HashMap<GameCondition, bool>,
    pub money_earned: i32,
}

impl StatsState {
    pub fn new() -> Self {
        let mut stats = HashMap::new();
        stats.insert(Stat::Sanity, 10);
        stats.insert(Stat::Food, 8);
        stats.insert(Stat::Gas, 8);
        stats.insert(Stat::Parts, 5);
        stats.insert(Stat::Money, 5);

        let conditions = HashMap::new();

        StatsState { stats, conditions, money_earned: 0, }
    }

    pub fn condition(&self, condition: GameCondition) -> bool {
        self.conditions.get(&condition).unwrap_or(&false).clone()
    }

    pub fn set_condition(&mut self, condition: GameCondition, val: bool) {
        let entry = self.conditions.entry(condition).or_insert(false);
        *entry = val;
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
        WriteExpect<'a, EventChannel<LogEvent>>,
    );

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);

        self.game_event_reader = Some(
            world
                .fetch_mut::<EventChannel<GameEvent>>()
                .register_reader(),
        );
    }

    fn run(&mut self, (game_events, mut stats, mut log_events): Self::SystemData) {
        // TODO
        // every 2 days, consume gasoline and flag generator as empty

        for event in game_events.read(&mut self.game_event_reader.as_mut().unwrap()) {
            match event {
                GameEvent::GameOver => {
                    stats.set_condition(GameCondition::GameOver, true);
                }
                GameEvent::NewDayStarted { day } => {
                    // If the lighthouse wasn't broken, add money to this paycheck
                    if stats.condition(GameCondition::LensBroken) || stats.condition(GameCondition::GeneratorBroken) {
                        log_events.single_write(LogEvent { message: String::from("The lighthouse wasn't on last night! Your pay will be docked."), color: COLOR_YELLOW });
                    } else {
                        stats.money_earned += 2;
                    }

                    if (day % 5) == 0 {
                        let amt = stats.money_earned;
                        stats.add(Stat::Money, amt);

                        if amt == 0 {
                            log_events.single_write(LogEvent { message: String::from("You didn't get a paycheck this week because the lighthouse has not been on."), color: COLOR_RED });
                        } else {
                            log_events.single_write(LogEvent { message: format!("You receive a paycheck for your duties. (Money +{})", amt), color: COLOR_GREEN });
                        }

                        stats.money_earned = 0;
                    }

                    stats.set_condition(GameCondition::FinalDay, *day >= 30);

                    // Handle food consumption
                    if !stats.condition(GameCondition::Starving) {
                        if stats.stat(Stat::Food) <= 0 {
                            stats.set_condition(GameCondition::Starving, true);
                            log_events.single_write(LogEvent { message: String::from("You are starving."), color: COLOR_RED });
                        } else {
                            log_events.single_write(LogEvent { message: String::from("You unpack the days rations from food storage. (Food -1)"), color: COLOR_BLACK });
                            stats.add(Stat::Food, -1);
                        }
                    } else {
                        if stats.stat(Stat::Food) <= 0 {
                            log_events.single_write(LogEvent { message: String::from("You collapse due to starvation."), color: COLOR_RED });
                            stats.set_condition(GameCondition::GameOver, true);
                            continue;
                        }

                        stats.set_condition(GameCondition::Starving, false);
                    }

                    // Handle sanity
                    if !stats.condition(GameCondition::Insane) {
                        if stats.stat(Stat::Sanity) <= 0 {
                            stats.set_condition(GameCondition::Insane, true);
                            println!("You can't make the voices stop.");
                            log_events.single_write(LogEvent { message: String::from("You can't make the voices stop."), color: COLOR_BLUE });
                        }
                    } else {
                        if stats.stat(Stat::Sanity) <= 0 {
                            log_events.single_write(LogEvent { message: String::from("In a fit of insanity, you throw yourself from atop the lighthouse."), color: COLOR_RED });
                            stats.set_condition(GameCondition::GameOver, true);
                            continue;
                        }

                        stats.set_condition(GameCondition::Insane, false);
                    }
                }
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
                GameEvent::HandleConditionEffects { effects } => {
                    for effect in effects {
                        match effect {
                            ConditionEffect::Set { condition } => {
                                stats.set_condition(*condition, true);
                                println!("SET {:?}", condition);
                            }
                            ConditionEffect::Clear { condition } => {
                                stats.set_condition(*condition, false);
                                println!("CLEAR {:?}", condition);
                            }
                        }
                    }
                }
                GameEvent::ActivityGoFishing => {
                    let roll: f32 = rand::thread_rng().gen();
                    if roll < 0.005 {
                        log_events.single_write(LogEvent { message: String::from("You catch a huge fish! (Food +2)"), color: COLOR_GREEN });
                        stats.add(Stat::Food, 2);
                    } else if roll <= 0.3 {
                        log_events.single_write(LogEvent { message: String::from("You catch a fish. (Food +1)"), color: COLOR_BLACK });
                        stats.add(Stat::Food, 1);
                    } else {
                        log_events.single_write(LogEvent { message: String::from("You try to catch a fish, but get no bites."), color: COLOR_RED });
                    }
                }
                _ => {}
            }
        }
    }
}

#[derive(Default)]
pub struct StatsInfoRenderSystem;

impl<'a> System<'a> for StatsInfoRenderSystem {
    type SystemData = (Write<'a, RenderState>, ReadExpect<'a, StatsState>);

    fn run(&mut self, (mut render, stats): Self::SystemData) {
        let icon_pos_x = 1225.0;
        // Sanity icon
        render.bind_transparency(Transparency::Opaque);
        render.bind_texture(resources::TEX_SPRITESHEET_UI);
        render.bind_color(COLOR_WHITE);
        render.bind_layer(layers::LAYER_UI);
        render.sprite(
            icon_pos_x,
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
            icon_pos_x,
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
            icon_pos_x,
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
            icon_pos_x,
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

        let text_pos_x = 1240.0;

        // Sanity text
        render.bind_texture(resources::TEX_FONT);
        render.bind_color(COLOR_BLACK);
        render.text(
            text_pos_x,
            64.0,
            8,
            16,
            1.0,
            &format!("{}", stats.stat(Stat::Sanity)),
        );

        // Food text
        render.text(
            text_pos_x,
            135.0,
            8,
            16,
            1.0,
            &format!("{}", stats.stat(Stat::Food)),
        );

        // Parts text
        render.text(
            text_pos_x,
            205.0,
            8,
            16,
            1.0,
            &format!("{}", stats.stat(Stat::Parts)),
        );

        // Gas text
        render.text(
            text_pos_x,
            270.0,
            8,
            16,
            1.0,
            &format!("{}", stats.stat(Stat::Gas)),
        );

        // Money text
        render.text(
            165.0,
            8.0,
            8,
            16,
            2.0,
            &format!("${}", stats.stat(Stat::Money)),
        );
    }
}
