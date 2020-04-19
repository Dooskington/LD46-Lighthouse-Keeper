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
}

impl MerchantState {
    pub fn new() -> Self {
        let next_arrival_day =
            rand::thread_rng().gen_range(MIN_MERCHANT_ARRIVAL_DAYS, MAX_MERCHANT_ARRIVAL_DAYS);
        MerchantState {
            has_arrived: false,
            next_arrival_day,
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
        ReadExpect<'a, EventChannel<GameEvent>>,
    );

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);

        self.game_event_reader = Some(
            world
                .fetch_mut::<EventChannel<GameEvent>>()
                .register_reader(),
        );
    }

    fn run(&mut self, (mut render, mut stats, input, mut merchant_state, game_events): Self::SystemData) {
        for event in game_events.read(&mut self.game_event_reader.as_mut().unwrap()) {
            match event {
                GameEvent::NewDayStarted { day } => {
                    if *day >= merchant_state.next_arrival_day {
                        merchant_state.has_arrived = true;
                        merchant_state.next_arrival_day = day + rand::thread_rng()
                        .gen_range(MIN_MERCHANT_ARRIVAL_DAYS, MAX_MERCHANT_ARRIVAL_DAYS);

                        println!("A merchant ship arrives, looking to sell some basic goods.");
                    }
                }
                GameEvent::NewTimeOfDayStarted { time_of_day } => {
                    if merchant_state.has_arrived && (*time_of_day == TimeOfDay::Night) {
                        println!("The merchant ship sails off into the sunset.");

                        merchant_state.has_arrived = false;
                    }
                }
                _ => {}
            }
        }

        if merchant_state.has_arrived {
            // TODO randomize these every arrival or something
            let food_price = 2;
            let gas_price = 3;
            let part_price = 4;

            // Window background
            render.bind_transparency(Transparency::Opaque);
            render.bind_texture(resources::TEX_SPRITESHEET_UI);
            render.bind_color(COLOR_WHITE);
            render.bind_layer(layers::LAYER_UI);
            render.sprite(
                0.0,
                200.0,
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
            render.text(16.0, 200.0 + 16.0, 8, 16, 1.5, "Merchant Ship");
            render.text(
                16.0,
                250.0,
                8,
                16,
                1.0,
                &format!("'1' => Purchase some food for ${}", food_price),
            );
            render.text(
                16.0,
                250.0 + 16.0,
                8,
                16,
                1.0,
                &format!("'2' => Purchase some gasoline for ${}", gas_price),
            );
            render.text(
                16.0,
                250.0 + 32.0,
                8,
                16,
                1.0,
                &format!("'3' => Purchase some parts for ${}", part_price),
            );

            render.text(
                16.0,
                350.0,
                8,
                16,
                1.0,
                "(Use keyboard)",
            );

            // Handle purchases
            let current_money = stats.stat(Stat::Money);
            if input.is_key_pressed(VirtualKeyCode::Key1) {
                if current_money >= food_price {
                    stats.add(Stat::Money, -food_price);
                    stats.add(Stat::Food, 1);
                    println!("You purchase some food.");
                } else {
                    println!("You don't have enough money for that...");
                }
            } else if input.is_key_pressed(VirtualKeyCode::Key2) {
                if current_money >= gas_price {
                    stats.add(Stat::Money, -gas_price);
                    stats.add(Stat::Gas, 1);
                    println!("You purchase some gasoline.");
                } else {
                    println!("You don't have enough money for that...");
                }
            } else if input.is_key_pressed(VirtualKeyCode::Key3) {
                if current_money >= part_price {
                    stats.add(Stat::Money, -part_price);
                    stats.add(Stat::Parts, 1);
                    println!("You purchase some parts.");
                } else {
                    println!("You don't have enough money for that...");
                }
            }
        }
    }
}
