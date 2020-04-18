use crate::game::{*, physics::*, Point2d};
use gfx::input::*;
use ncollide2d::pipeline::CollisionGroups;
use specs::prelude::*;

pub struct AlienComponent {
    pub nutrient_a: f32,
    pub nutrient_b: f32,
    pub nutrient_c: f32,
    pub nutrient_d: f32,
    pub temperature: f32,
}

impl AlienComponent {
    pub fn new() -> Self {
        AlienComponent {
            nutrient_a: 75.0,
            nutrient_b: 50.0,
            nutrient_c: 25.0,
            nutrient_d: 45.0,
            temperature: 8.0,
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
        let temp_adjust_speed = 0.25;

        for alien in (&mut aliens).join() {
            alien.temperature = lerp(alien.temperature, workstation.room_temperature, temp_adjust_speed * 0.016);
        }
    }
}

#[derive(Default)]
pub struct AlienInfoRenderSystem;

impl<'a> System<'a> for AlienInfoRenderSystem {
    type SystemData = (
        Write<'a, RenderState>,
        ReadStorage<'a, AlienComponent>,
    );

    fn run(&mut self, (mut render, aliens): Self::SystemData) {
        for alien in (&aliens).join() {
            render.bind_layer(layers::LAYER_UI);
            render.bind_transparency(Transparency::Transparent);
            render.bind_texture(resources::TEX_FONT);

            render.bind_color(COLOR_BLUE);
            render.text(8.0, 65.0, 8, 16, 1.1, "NUTRIENTS");
            render.text(8.0, 86.0, 8, 16, 1.0, &format!("A: {:.1}", alien.nutrient_a));
            render.text(8.0, 102.0, 8, 16, 1.0, &format!("B: {:.1}", alien.nutrient_b));
            render.text(8.0, 118.0, 8, 16, 1.0, &format!("C: {:.1}", alien.nutrient_c));
            render.text(8.0, 134.0, 8, 16, 1.0, &format!("D: {:.1}", alien.nutrient_d));

            render.bind_color(COLOR_RED);
            render.text(8.0, 155.0, 8, 16, 1.1, &format!("TEMP: {:.1}", alien.temperature));
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