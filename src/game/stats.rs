use crate::game::*;
use specs::prelude::*;

pub struct StatsState {
    sanity: i32,
    food: i32,
    gas: i32,
    parts: i32,
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
pub struct StatsInfoRenderSystem;

impl<'a> System<'a> for StatsInfoRenderSystem {
    type SystemData = (Write<'a, RenderState>, ReadExpect<'a, StatsState>);

    fn run(&mut self, (mut render, stats): Self::SystemData) {
        /*
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
                h: 160
            },
        );

        let hours_bar_region = match time.hours_passed {
            0 => SpriteRegion {
                x: 320,
                y: 0,
                w: 288,
                h: 64
            },
            1 => SpriteRegion {
                x: 320,
                y: 64,
                w: 288,
                h: 64
            },
            2 => SpriteRegion {
                x: 320,
                y: 128,
                w: 288,
                h: 64
            },
            3 => SpriteRegion {
                x: 320,
                y: 192,
                w: 288,
                h: 64
            },
            _ => SpriteRegion {
                x: 320,
                y: 256,
                w: 288,
                h: 64
            },
        };

        render.sprite(
            0.0,
            73.0,
            Point2f::origin(),
            Vector2f::new(0.5, 0.5),
            hours_bar_region,
        );

        // Day Text
        render.bind_layer(layers::LAYER_UI);
        render.bind_transparency(Transparency::Opaque);
        render.bind_texture(resources::TEX_FONT);
        render.bind_color(COLOR_BLACK);
        render.text(
            8.0,
            8.0,
            8,
            16,
            2.0,
            &format!("Day {}", time.day),
        );

        render.text(
            8.0,
            48.0,
            8,
            16,
            1.5,
            &format!("{}", time.time_of_day),
        );
        */
    }
}
