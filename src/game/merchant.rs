use crate::game::{physics::*, Point2d, *};
use gfx::input::*;
use ncollide2d::pipeline::CollisionGroups;
use rand::Rng;
use specs::prelude::*;

const MIN_MERCHANT_ARRIVAL_DAYS: i32 = 4;
const MAX_MERCHANT_ARRIVAL_DAYS: i32 = 7;

#[derive(Default)]
pub struct MerchantState {
    has_arrived: bool,
    next_arrival_day: i32,
    food_price: i32,
    gas_price: i32,
    part_price: i32,
}

impl MerchantState {
    pub fn new() -> Self {
        let next_arrival_day =
            rand::thread_rng().gen_range(MIN_MERCHANT_ARRIVAL_DAYS, MAX_MERCHANT_ARRIVAL_DAYS);
        MerchantState {
            has_arrived: false,
            next_arrival_day,
            food_price: 2,
            gas_price: 3,
            part_price: 4,
        }
    }
}

#[derive(Default)]
pub struct MerchantSystem {
    game_event_reader: Option<ReaderId<GameEvent>>,
}

impl<'a> System<'a> for MerchantSystem {
    type SystemData = (
        WriteExpect<'a, RenderState>,
        WriteExpect<'a, StatsState>,
        ReadExpect<'a, InputState>,
        WriteExpect<'a, MerchantState>,
        WriteExpect<'a, EventChannel<GameEvent>>,
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

    fn run(&mut self, (mut render, mut stats, input, mut merchant_state, mut game_events, mut log_events): Self::SystemData) {
        for event in game_events.read(&mut self.game_event_reader.as_mut().unwrap()) {
            match event {
                GameEvent::NewDayStarted { day } => {
                    if *day >= merchant_state.next_arrival_day {
                        merchant_state.has_arrived = true;
                        merchant_state.next_arrival_day = day + rand::thread_rng()
                        .gen_range(MIN_MERCHANT_ARRIVAL_DAYS, MAX_MERCHANT_ARRIVAL_DAYS);

                        log_events.single_write(LogEvent { message: String::from("A merchant ship arrives, looking to sell some basic goods."), color: COLOR_YELLOW });
                    }
                }
                GameEvent::NewTimeOfDayStarted { time_of_day } => {
                    if merchant_state.has_arrived && (*time_of_day == TimeOfDay::Night) {
                        log_events.single_write(LogEvent { message: String::from("The merchant ship sails off into the sunset."), color: COLOR_YELLOW });

                        let mut rand = rand::thread_rng();
                        merchant_state.food_price = rand.gen_range(2, 4);
                        merchant_state.gas_price = rand.gen_range(3, 8);
                        merchant_state.part_price = rand.gen_range(3, 10);

                        merchant_state.has_arrived = false;
                    }
                }
                _ => {}
            }
        }

        if stats.condition(GameCondition::GameOver) {
            return;
        }

        if merchant_state.has_arrived {
            let pos_x = 0.0;
            let pos_y = 500.0;

            // Window background
            render.bind_transparency(Transparency::Opaque);
            render.bind_texture(resources::TEX_SPRITESHEET_UI);
            render.bind_color(COLOR_WHITE);
            render.bind_layer(layers::LAYER_UI);
            render.sprite(
                pos_x,
                pos_y,
                Point2f::new(0.5, 0.5),
                Vector2f::new(2.0, 2.0),
                SpriteRegion {
                    x: 0,
                    y: 160,
                    w: 160,
                    h: 96,
                },
            );

            // Render text and prices for shop items
            render.bind_texture(resources::TEX_FONT);
            render.bind_color(COLOR_BLACK);
            render.text(pos_x + 16.0, pos_y + 16.0, 8, 16, 1.5, "Merchant Ship");
            render.text(
                pos_x + 16.0,
                pos_y + 50.0,
                8,
                16,
                1.0,
                &format!("'1' => Purchase some food for ${}", merchant_state.food_price),
            );
            render.text(
                pos_x + 16.0,
                pos_y + 50.0 + 16.0,
                8,
                16,
                1.0,
                &format!("'2' => Purchase some gasoline for ${}", merchant_state.gas_price),
            );
            render.text(
                pos_x + 16.0,
                pos_y + 50.0 + 32.0,
                8,
                16,
                1.0,
                &format!("'3' => Purchase some parts for ${}", merchant_state.part_price),
            );

            render.text(
                pos_x + 16.0,
                pos_y + 50.0 + 48.0,
                8,
                16,
                1.0,
                "(Use keyboard)",
            );

            // Handle purchases
            let mut did_purchase = false;
            let current_money = stats.stat(Stat::Money);
            if input.is_key_pressed(VirtualKeyCode::Key1) {
                if current_money >= merchant_state.food_price {
                    stats.add(Stat::Money, -merchant_state.food_price);
                    stats.add(Stat::Food, 1);
                    did_purchase = true;
                    log_events.single_write(LogEvent { message: String::from("You purchase some food."), color: COLOR_GREEN });
                } else {
                    log_events.single_write(LogEvent { message: String::from("You don't have enough money for that..."), color: COLOR_RED });
                }
            } else if input.is_key_pressed(VirtualKeyCode::Key2) {
                if current_money >= merchant_state.gas_price {
                    stats.add(Stat::Money, -merchant_state.gas_price);
                    stats.add(Stat::Gas, 1);
                    did_purchase = true;
                    log_events.single_write(LogEvent { message: String::from("You purchase some gas."), color: COLOR_GREEN });
                } else {
                    log_events.single_write(LogEvent { message: String::from("You don't have enough money for that..."), color: COLOR_RED });
                }
            } else if input.is_key_pressed(VirtualKeyCode::Key3) {
                if current_money >= merchant_state.part_price {
                    stats.add(Stat::Money, -merchant_state.part_price);
                    stats.add(Stat::Parts, 1);
                    did_purchase = true;
                    log_events.single_write(LogEvent { message: String::from("You purchase some parts."), color: COLOR_GREEN });
                } else {
                    log_events.single_write(LogEvent { message: String::from("You don't have enough money for that..."), color: COLOR_RED });
                }
            }

            if did_purchase {
                game_events.single_write(GameEvent::RefreshActivities);
            }
        }
    }
}
