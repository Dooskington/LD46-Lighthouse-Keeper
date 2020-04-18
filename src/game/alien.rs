use crate::game::{physics::*, Point2d, *};
use gfx::input::*;
use ncollide2d::pipeline::CollisionGroups;
use specs::prelude::*;

pub const MIN_DESIRED_NUTRIENT_A: f32 = 85.0;
pub const DESIRED_NUTRIENT_A: f32 = 90.0;
pub const MIN_DESIRED_NUTRIENT_B: f32 = 45.0;
pub const MAX_DESIRED_NUTRIENT_B: f32 = 55.0;
pub const DESIRED_NUTRIENT_B: f32 = 50.0;
pub const MAX_NUTRIENT: f32 = 100.0;
pub const MIN_TEMPERATURE: f32 = -10.0;
pub const MAX_TEMPERATURE: f32 = 20.0;
pub const DESIRED_TEMPERATURE: f32 = 10.0;
pub const DESIRED_TEMPERATURE_DIFF: f32 = 8.0;

pub enum AlienHealthState {
    Unknown,
    Healthy,
    Stable,
    Sickly,
    Critical,
}

impl AlienHealthState {
    pub fn from_stats(nutrient_a: f32, nutrient_b: f32, temperature: f32) -> Self {
        let health_possibilities = vec![
            (AlienHealthState::Healthy, DESIRED_NUTRIENT_A, DESIRED_NUTRIENT_B, 0.0),
            (AlienHealthState::Stable, DESIRED_NUTRIENT_A * 0.9, DESIRED_NUTRIENT_B * 0.9, DESIRED_TEMPERATURE_DIFF * 0.5),
            (AlienHealthState::Sickly, DESIRED_NUTRIENT_A * 0.75, DESIRED_NUTRIENT_B * 0.75, DESIRED_TEMPERATURE_DIFF * 1.25),
            (AlienHealthState::Critical, DESIRED_NUTRIENT_A * 0.5, DESIRED_NUTRIENT_B * 0.5, DESIRED_TEMPERATURE_DIFF * 1.5),
        ];

        let temperature_difference = (temperature - DESIRED_TEMPERATURE).abs();
        //println!("nutrient_a: {}, nutrient_b: {}, temperature_difference: {}", nutrient_a, nutrient_b, temperature_difference);

        let mut selected = AlienHealthState::Unknown;
        let mut lowest_dist = std::f32::MAX;
        for possibility in health_possibilities {
            let nutrient_a_dist = (nutrient_a - possibility.1).abs() / MAX_NUTRIENT;
            let nutrient_b_dist = (nutrient_b - possibility.2).abs() / MAX_NUTRIENT;
            let temp_diff_dist = (temperature_difference - possibility.3).abs() / (MAX_TEMPERATURE - MIN_TEMPERATURE);
            let dist = nutrient_a_dist + nutrient_b_dist + (temp_diff_dist * 2.25);
            //println!("{}, {}, {}, total {} ({})", nutrient_a_dist, nutrient_b_dist, temp_diff_dist, dist, possibility.0);
            if dist < lowest_dist {
                lowest_dist = dist;
                selected = possibility.0;
            }
        }

        selected
    }
}

impl std::fmt::Display for AlienHealthState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let printable = match *self {
            AlienHealthState::Healthy => "Healthy",
            AlienHealthState::Stable => "Stable",
            AlienHealthState::Sickly => "Sickly",
            AlienHealthState::Critical => "Critical",
            _ => "Unknown",
        };

        write!(f, "{}", printable)
    }
}

pub struct AlienComponent {
    pub nutrient_a: f32,
    pub nutrient_b: f32,
    pub nutrient_c: f32,
    pub nutrient_d: f32,
    pub temperature: f32,
    pub health: AlienHealthState,
}

impl AlienComponent {
    pub fn new() -> Self {
        AlienComponent {
            nutrient_a: 75.0,
            nutrient_b: 50.0,
            nutrient_c: 25.0,
            nutrient_d: 45.0,
            temperature: 8.0,
            health: AlienHealthState::Unknown,
        }
    }
}

impl Component for AlienComponent {
    type Storage = VecStorage<Self>;
}

#[derive(Default)]
pub struct AlienSystem;

impl<'a> System<'a> for AlienSystem {
    type SystemData = (
        ReadExpect<'a, WorkstationState>,
        WriteStorage<'a, AlienComponent>,
    );

    fn run(&mut self, (workstation, mut aliens): Self::SystemData) {
        let temp_adjust_speed = 0.05;

        for alien in (&mut aliens).join() {
            alien.temperature = lerp(
                alien.temperature,
                workstation.room_temperature,
                temp_adjust_speed * 0.016,
            );

            alien.health = AlienHealthState::from_stats(alien.nutrient_a, alien.nutrient_b, alien.temperature)
        }
    }
}

#[derive(Default)]
pub struct AlienInfoRenderSystem;

impl<'a> System<'a> for AlienInfoRenderSystem {
    type SystemData = (Write<'a, RenderState>, ReadStorage<'a, AlienComponent>);

    fn run(&mut self, (mut render, aliens): Self::SystemData) {
        for alien in (&aliens).join() {
            render.bind_layer(layers::LAYER_UI);
            render.bind_transparency(Transparency::Transparent);
            render.bind_texture(resources::TEX_FONT);

            let health_color = match alien.health {
                AlienHealthState::Healthy => COLOR_GREEN,
                AlienHealthState::Stable => COLOR_GREEN,
                AlienHealthState::Sickly => COLOR_YELLOW,
                AlienHealthState::Critical => COLOR_RED,
                AlienHealthState::Unknown => COLOR_WHITE,
            };

            render.bind_color(health_color);
            render.text(100.0, 65.0, 8, 16, 1.1, &format!("STATUS: {}", alien.health));

            render.bind_color(COLOR_BLUE);
            render.text(8.0, 65.0, 8, 16, 1.1, "NUTRIENTS");

            let nutrient_a_percentage = if alien.nutrient_a < MIN_DESIRED_NUTRIENT_A {
                let dist = MIN_DESIRED_NUTRIENT_A - alien.nutrient_a;
                1.0 - (dist / MIN_DESIRED_NUTRIENT_A)
            } else {
                1.0
            };

            let nutrient_a_color = color_lerp(COLOR_RED, COLOR_GREEN, nutrient_a_percentage);
            render.bind_color(nutrient_a_color);
            render.text(
                8.0,
                86.0,
                8,
                16,
                1.0,
                &format!("A: {:.1}", alien.nutrient_a),
            );

            let nutrient_b_percentage = {
                if alien.nutrient_b < MIN_DESIRED_NUTRIENT_B {
                    let dist = MIN_DESIRED_NUTRIENT_B - alien.nutrient_b;
                    1.0 - (dist / MIN_DESIRED_NUTRIENT_B)
                } else if alien.nutrient_b > MAX_DESIRED_NUTRIENT_B {
                    let dist = alien.nutrient_b - MAX_DESIRED_NUTRIENT_B;
                    1.0 - (dist / (MAX_NUTRIENT - MAX_DESIRED_NUTRIENT_B))
                } else {
                    1.0
                }
            };

            let nutrient_b_color = color_lerp(COLOR_RED, COLOR_GREEN, nutrient_b_percentage);
            render.bind_color(nutrient_b_color);
            render.text(
                8.0,
                102.0,
                8,
                16,
                1.0,
                &format!("B: {:.1}", alien.nutrient_b),
            );

            render.bind_color(COLOR_GREEN);
            render.text(
                8.0,
                118.0,
                8,
                16,
                1.0,
                &format!("C: {:.1}", alien.nutrient_c),
            );
            render.bind_color(COLOR_GREEN);
            render.text(
                8.0,
                134.0,
                8,
                16,
                1.0,
                &format!("D: {:.1}", alien.nutrient_d),
            );

            let temp_percentage = {
                let temp_diff = (alien.temperature - DESIRED_TEMPERATURE).abs();
                1.0 - (temp_diff / DESIRED_TEMPERATURE_DIFF)
            };

            let temp_color = color_lerp(COLOR_RED, COLOR_GREEN, temp_percentage);
            render.bind_color(temp_color);
            render.text(
                8.0,
                155.0,
                8,
                16,
                1.1,
                &format!("TEMP: {:.1}", alien.temperature),
            );
        }
    }
}

// How will this work?
// AlienComponent has stats
// nutrients: AlienNutrients { nutrient_a, nutrient_b, nutrient_c, nutrient_d }
// temperature: TemperatureCelsius
// desired_temperature: TemperatureCelsius
// health: u32

// AlienSystem needs to affect stats over time
// nutrient a needs to be kept high. drains quickly, decaying (by half) into nutrient b
// nutrient b needs to be kept in the middle of the range. doesn't drain naturally. made from nutrient a.
// nutrient c varies on its own. It modifies how effective nutrient d is
// nutrient d regulates nutrients. A nutrient regulation can be triggerd by administiring 50 units of various catalysts
// health or happiness derived from some point cloud with those 4 nutrients?
// temperature will slowly move to match room temperature

// How are the stats shown to the player
// For now, just use text and colors
// AlienInfoRenderSystem?
